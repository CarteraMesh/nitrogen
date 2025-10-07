use {
    alloy_primitives::hex,
    anyhow::{Result, format_err},
    serde::Deserialize,
    std::{thread::sleep, time::Duration},
    tracing::{debug, error, info, trace},
};
const IRIS_API_URL: &str = "https://iris-api-sandbox.circle.com";

/// The bytes of the attestation.
pub type AttestationBytes = Vec<u8>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttestationResponse {
    pub messages: Vec<AttestationMessage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttestationMessage {
    pub status: AttestationStatus,
    #[serde(default)]
    pub attestation: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

/// Represents the status of the attestation.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum AttestationStatus {
    Complete,
    Pending,
    PendingConfirmations,
    Failed,
}

pub async fn get_attestation_with_retry(
    message_hash: String,
    max_attempts: Option<u32>,
    poll_interval: Option<u64>,
) -> Result<(AttestationBytes, AttestationBytes)> {
    let client = reqwest::Client::new();
    let max_attempts = max_attempts.unwrap_or(30);
    let poll_interval = poll_interval.unwrap_or(10);

    info!(message_hash = ?message_hash, "Polling for attestation ...");

    let url = format!("{IRIS_API_URL}/v2/messages/5?transactionHash={message_hash}");

    info!(url = ?url, "Attestation URL");

    for attempt in 1..=max_attempts {
        info!(
            attempt = ?attempt,
            max_attempts = ?max_attempts,
            "Getting attestation ..."
        );
        let response = client.get(&url).send().await?;
        trace!(attestation_status = ?response.status());

        // Handle rate limiting
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let secs = 5 * 60;
            info!(sleep_secs = ?secs, "Rate limit exceeded, waiting before retrying");
            sleep(Duration::from_secs(secs));
            continue;
        }

        // Handle 404 status - treat as pending since the attestation likely doesn't
        // exist yet
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            info!(
                attempt = ?attempt,
                max_attempts = ?max_attempts,
                poll_interval = ?poll_interval,
                "Attestation not found (404), waiting before retrying"
            );
            sleep(Duration::from_secs(poll_interval));
            continue;
        }

        // Ensure the response status is successful before trying to parse JSON
        response.error_for_status_ref()?;

        info!("Decoding attestation response");

        let attestation: AttestationResponse = match response.json::<serde_json::Value>().await {
            Ok(attestation) => {
                debug!(attestation = ?attestation, "Attestation response");
                serde_json::from_value(attestation)?
            }
            Err(e) => {
                error!(error = ?e, "Error decoding attestation response");
                continue;
            }
        };
        if attestation.messages.is_empty() {
            return Err(format_err!("Attestation response is empty"))?;
        }
        let message = attestation.messages.into_iter().next().unwrap();
        match message.status {
            AttestationStatus::Complete => {
                let attestation_bytes = message
                    .attestation
                    .ok_or_else(|| format_err!("Attestation missing".to_string()))?;

                let attestation_message = message
                    .message
                    .ok_or_else(|| format_err!("Attestation missing".to_string()))?;
                // Remove '0x' prefix if present and decode hex
                let attestation_bytes = if let Some(stripped) = attestation_bytes.strip_prefix("0x")
                {
                    hex::decode(stripped)
                } else {
                    hex::decode(&attestation_bytes)
                }?;

                let attestation_message =
                    if let Some(stripped) = attestation_message.strip_prefix("0x") {
                        hex::decode(stripped)
                    } else {
                        hex::decode(&attestation_message)
                    }?;

                info!("Attestation received successfully");
                return Ok((attestation_bytes, attestation_message));
            }
            AttestationStatus::Failed => {
                return Err(format_err!("Attestation failed".to_string()))?;
            }
            AttestationStatus::Pending | AttestationStatus::PendingConfirmations => {
                info!(
                    attempt = ?attempt,
                    max_attempts = ?max_attempts,
                    poll_interval = ?poll_interval,
                    "Attestation pending, waiting before retrying"
                );
                sleep(Duration::from_secs(poll_interval));
            }
        }
    }

    Err(format_err!("timeout"))
}
