use crate::prelude::*;
use dominion::prelude::*;

use std::sync::{Arc, Mutex};

pub async fn handle_client_message(msg: ClientMessage, data: Arc<Mutex<Game>>, player_number: usize, callbacks: &Box<dyn Callbacks>, message_channels: &mut ServerMessageChannels) {
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
        ClientMessage::NextPhase => {
            let data = data.clone();
            next_phase(data, player_number, callbacks, message_channels).await;
        }
        ClientMessage::PlayCard { index } => {
            let data = data.clone();
            play_card(data, player_number, index, callbacks, message_channels).await;
        }
        ClientMessage::BuyCard { card } => {
            let data = data.clone();
            buy_card(data, player_number, &card, callbacks, message_channels).await;
        }

        _ => {
            println!("Server received an unknown message from the client!");
            println!("Message: {:?}", msg);
        }
    }
}

pub async fn next_phase(data: Arc<Mutex<Game>>, player_number: usize, callbacks: &Box<dyn Callbacks>, message_channels: &mut ServerMessageChannels) {
    let (current_turn, player, phase);
    {
        let game = data.lock().unwrap();
        current_turn = game.current_turn;
    }

    if current_turn != player_number {
        message_channels.value_sender.send(serde_json::to_value(ServerMessage::NotYourTurn).unwrap()).await.unwrap();
        return;
    }

    {
        let mut game = data.lock().unwrap();
        player = &mut game.players[player_number];
        phase = player.phase;

        match phase {
            Phase::ActionPhase => {
                player.phase = Phase::BuyPhase;
            }
            Phase::OutOfTurn => panic!(),
            Phase::BuyPhase => {
                player.phase = Phase::NightPhase;
            }
            Phase::NightPhase => {
                player.phase = Phase::CleanupPhase;
                player.cleanup();
                player.phase = Phase::OutOfTurn;
            }
            Phase::CleanupPhase => panic!(),
            _ => todo!(),
        }
    }
}

pub async fn play_card(data: Arc<Mutex<Game>>, player_number: usize, card_index: usize, callbacks: &Box<dyn Callbacks>, message_channels: &mut ServerMessageChannels) {
    let (current_turn, player, phase, card);
    {
        let game = data.lock().unwrap();
        current_turn = game.current_turn;
        player = &game.players[player_number];
        phase = player.phase;
        card = player.hand[card_index].clone();
    }

    if current_turn != player_number {
        message_channels.value_sender.send(serde_json::to_value(ServerMessage::NotYourTurn).unwrap()).await.unwrap();
        return;
    }

    match phase {
        Phase::ActionPhase => {
            if !card.is_action() {
                message_channels.value_sender.send(
                    serde_json::to_value(
                        ServerMessage::IllegalPlay {
                            card: card.clone(),
                            reason: DominionError::WrongPhase
                        }
                    ).unwrap()).await.unwrap();

                return;
            }

            let mut game = data.lock().unwrap();
            // TODO: check reactions

            game.play_action_from_hand(player_number, card_index, callbacks).unwrap();

        }
        Phase::BuyPhase => {
            if !card.is_treasure() {
                message_channels.value_sender.send(
                    serde_json::to_value(
                        ServerMessage::IllegalPlay {
                            card: card.clone(),
                            reason: DominionError::CardTypeMisMatch { expected: Treasure }
                        }
                    ).unwrap()).await.unwrap();

                return;
            }

            {
                let mut game = data.lock().unwrap();

                // We know the card is a treasure card, so unwrap
                game.play_treasure(player_number, card_index, callbacks).unwrap();
            }
        }
        _ => {
            message_channels.value_sender.send(
                serde_json::to_value(
                    ServerMessage::WrongPhase
                ).unwrap()).await.unwrap();

            return;
        }
    }

    println!("Player {} played {}!", player_number, card.name());
}

pub async fn buy_card(data: Arc<Mutex<Game>>, player_number: usize, card: &Box<dyn Card>, callbacks: &Box<dyn Callbacks>, message_channels: &mut ServerMessageChannels) {
    let (current_turn, player, phase);
    {
        let game = data.lock().unwrap();
        current_turn = game.current_turn;
        player = &game.players[player_number];
        phase = player.phase;
    }

    if current_turn != player_number {
        message_channels.value_sender.send(serde_json::to_value(ServerMessage::NotYourTurn).unwrap()).await.unwrap();
        return;
    }

    match phase {
        Phase::BuyPhase => {

        }
        _ => {
            message_channels.value_sender.send(
                serde_json::to_value(
                    ServerMessage::WrongPhase
                ).unwrap()).await.unwrap();

            return;
        }
    }

    println!("Player {} bought {}!", player_number, card.name());
}
