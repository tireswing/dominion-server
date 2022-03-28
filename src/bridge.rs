use dominion::prelude::*;
use crate::prelude::*;

use std::{mem::{self, Discriminant}, sync::{Arc, Mutex}};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub struct ServerBridge {
    pub sender: BroadcastSender,
    pub waiting_receivers: Arc<Mutex<Vec<BridgeResponseWrapper>>>,
}

#[derive(Clone, Debug)]
pub struct BridgeResponseWrapper {
    expected_response: Discriminant<ClientMessage>,
    player_number: usize,
    mailbox: mpsc::Sender<BridgeResponse>
}

#[derive(Clone, Debug)]
pub enum BridgeResponse {
    Card { card: Option<Box<dyn Card>> }
}

#[allow(unused_variables)]
impl Callbacks for ServerBridge {
    fn choose_card_from_supply(&self, player_number: usize) -> Option<Box<dyn Card>> {
        let (tx, mut rx) = mpsc::channel::<BridgeResponse>(1);
        let response = BridgeResponseWrapper {
            expected_response: mem::discriminant(&ClientMessage::ChooseCard{ card: None }),
            player_number,
            mailbox: tx,
        };
        let message = serde_json::to_value(ServerMessage::ChooseCardFromSupply).unwrap();
        let recipients = Recipients::SingleRecipient { recipient: player_number };
        self.sender.send((message, recipients));

        let response = rx.blocking_recv().unwrap();
        rx.close();

        match response {
            BridgeResponse::Card { card } => {
                card
            }

            _ => {
                panic!("Bad BridgeResponse type!")
            }
        }
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
