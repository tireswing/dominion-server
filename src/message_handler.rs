use crate::prelude::*;
use dominion::prelude::*;

use std::sync::{Arc, Mutex};

pub async fn handle_client_message(msg: ClientMessage, data: Arc<Mutex<Game>>, player_number: usize, message_channels: &mut ServerMessageChannels) {
    match msg {
        ClientMessage::Ping => {
            println!("Got a ping from player {}!", player_number);
            message_channels.value_sender.send(serde_json::to_value(ServerMessage::PingResponse).unwrap()).await.unwrap();
        }
        ClientMessage::ChatMessage{ message } => {
            let game = data.lock().unwrap();
            let player_count = game.player_count();
            // let sender = &game.players[player_number];
            // let author = sender.uuid;
            let message = serde_json::to_value(ServerMessage::ChatMessage{ author: player_number, message }).unwrap();
            let recipients = everyone_but(player_count, player_number);
            message_channels.broadcast_sender.send((message, recipients)).unwrap();
        }
        ClientMessage::StartGame { supply_list } => {
            let mut game = data.lock().unwrap();
            if game.started {
                let recipients = Recipients::SingleRecipient { recipient: player_number };
                let message = serde_json::to_value(ServerMessage::GameAlreadyStarted).unwrap();
                message_channels.broadcast_sender.send((message, recipients)).unwrap();
                return;
            }

            match game.generate_supply(supply_list) {
                Ok(()) => {
                    game.started = true;
                    for player_number in 0..game.player_count() {
                        let recipients = Recipients::SingleRecipient { recipient: player_number };
                        let state = game.partial_game(player_number);
                        let message = serde_json::to_value(ServerMessage::StartingGame { state }).unwrap();
                        message_channels.broadcast_sender.send((message, recipients)).unwrap();
                    }
                }
                Err(NotEnoughPlayers) => {
                    let recipients = Recipients::SingleRecipient { recipient: player_number };
                    let message = serde_json::to_value(ServerMessage::NotEnoughPlayers).unwrap();
                    message_channels.broadcast_sender.send((message, recipients)).unwrap();
                }
                Err(e) => {
                    panic!("Unknown error while starting: {:?}", e);
                }
            }
        }
        ClientMessage::PlayCard { index } => {
            let data = data.clone();
            play_card(data, player_number, index, message_channels).await;
        }

        _ => {
            println!("Server received an unknown message from the client!");
            println!("Message: {:?}", msg);
        }
    }
}

pub async fn play_card(data: Arc<Mutex<Game>>, player_number: usize, card_index: usize, message_channels: &mut ServerMessageChannels) {
    let (current_turn, player, phase, card);
    {
        let game = data.lock().unwrap();
        current_turn = game.current_turn;
        player = &game.players[player_number];
        phase = player.phase;
        card = player.hand[card_index].clone();
    }

    if current_turn != player_number {
        value_sender.send(serde_json::to_value(ServerMessage::NotYourTurn).unwrap()).await.unwrap();
        return;
    }

    match phase {
        Phase::ActionPhase => {
            if !card.is_action() {
                message_channels.value_sender.send(
                    serde_json::to_value(
                        ServerMessage::IllegalPlay {
                            card: card.clone(),
                            reason: IllegalPlayReason::WrongPhase
                        }
                    ).unwrap()).await.unwrap();
            }

            let game = data.lock().unwrap();
            // TODO: check reactions

            game.play_action_from_hand(player_number, card_index);

        }
        Phase::BuyPhase => {
            if !card.is_treasure() {
                message_channels.value_sender.send(serde_json::to_value(
                    ServerMessage::IllegalPlay {
                        card: card.clone(),
                        reason: IllegalPlayReason::WrongPhase
                    }).unwrap()).await.unwrap();
            }
        }
        _ => {}
    }

    println!("Player {} played {}!", player_number, card.name());
}
