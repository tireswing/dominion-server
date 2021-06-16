use crate::prelude::*;
use dominion::prelude::*;

use std::sync::{Arc, Mutex};

pub async fn play_card(data: Arc<Mutex<Game>>, player_number: usize, card_index: usize, serialized: &mut Serialized) {
    let (current_turn, player, phase, card);
    {
        let game = data.lock().unwrap();
        current_turn = game.current_turn;
        player = &game.players[player_number];
        phase = player.phase;
        card = player.hand[card_index].clone();
    }

    if current_turn != player_number {
        serialized.send(serde_json::to_value(ServerMessage::NotYourTurn).unwrap()).await.unwrap();
        return;
    }

    match phase {
        Phase::ActionPhase => {
            if !card.is_action() {
                serialized.send(serde_json::to_value(ServerMessage::IllegalPlay { card: card.clone(), reason: IllegalPlayReason::WrongPhase }).unwrap()).await.unwrap();
            }
        }
        Phase::BuyPhase => {
            if !card.is_treasure() {
                serialized.send(serde_json::to_value(ServerMessage::IllegalPlay { card: card.clone(), reason: IllegalPlayReason::WrongPhase }).unwrap()).await.unwrap();
            }
        }
        _ => {}
    }

    println!("Player {} played {}!", player_number, card.name());
}
