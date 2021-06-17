use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Recipients {
    SingleRecipient { recipient: usize },
    MultipleRecipients { recipients: Vec<usize> },
    Everyone,
}

pub fn everyone_but(player_count: usize, player_number: usize) -> Recipients {
    let mut recipients = (0..player_count).collect::<Vec<usize>>();
    recipients.remove(player_number);

    Recipients::MultipleRecipients { recipients }
}
