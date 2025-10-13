pub mod receive_message;
pub mod reclaim_event_account;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub enum MessageTransmitterV2Instruction {
    ReceiveMessage(receive_message::ReceiveMessage),
    ReclaimEventAccount(reclaim_event_account::ReclaimEventAccount),
}

pub fn receive_message(
    params: crate::types::ReceiveMessageParams,
) -> receive_message::ReceiveMessage {
    receive_message::ReceiveMessage { params }
}

pub fn reclaim_event_account(
    params: crate::types::ReclaimEventAccountParams,
) -> reclaim_event_account::ReclaimEventAccount {
    reclaim_event_account::ReclaimEventAccount { params }
}
