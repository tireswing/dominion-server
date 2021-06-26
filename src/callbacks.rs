use dominion::prelude::*;
use crate::prelude::*;

use std::mem::{self, Discriminant};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub struct ServerBridge {
    pub sender: BroadcastSender,
}

pub struct BridgeResponse<T> {
    expected_response: Discriminant<ClientMessage>,
    player_number: usize,
    mailbox: mpsc::Sender<T>
}

#[allow(unused_variables)]
impl Callbacks for ServerBridge {
    fn choose_card_from_supply(&self, player_number: usize) -> Option<Box<dyn Card>> {
        let (tx, mut rx) = mpsc::channel::<Option<Box<dyn Card>>>(1);
        let response = BridgeResponse {
            expected_response: mem::discriminant(&ClientMessage::ChooseCard{ card: None }),
            player_number,
            mailbox: tx,
        };
        let message = serde_json::to_value(ServerMessage::ChooseCardFromSupply).unwrap();
        let recipients = Recipients::SingleRecipient { recipient: player_number };
        self.sender.send((message, recipients));

        let event = rx.blocking_recv().unwrap();
        rx.close();
        event
    }
    fn choose_card_from_hand(&self, message: &str) -> usize {
        todo!();
    }
    fn choose_card_from_hand_opt(&self, message: &str) -> Option<usize> {
        todo!();
    }
    fn choose_cards_from_hand(&self, number_to_choose: usize, message: &str) -> Vec<usize> {
        todo!();
    }
    fn reveal_hand(&self, player_number: usize) {
        todo!();
    }
    fn get_player_consent(&self, player_number: usize, prompt: &str) -> bool {
        todo!();
    }
}
