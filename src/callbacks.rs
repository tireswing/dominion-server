use dominion::prelude::*;

#[derive(Clone, Debug)]
pub struct ServerBridge;

impl ServerBridge {
    pub fn new() -> ServerBridge {
        ServerBridge
    }
}

impl Callbacks for ServerBridge {
    fn choose_card_from_supply(&self, supply: &Supply) -> Option<Box<dyn Card>> {
        todo!();
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
