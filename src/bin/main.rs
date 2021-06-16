use dominion::prelude::*;
use dominion_server::prelude::*;

use std::sync::{Arc, Mutex};

use anyhow::Result;
use serde_json::Value;
use tokio::{net::{TcpListener, TcpStream}, sync::broadcast};
use tokio_serde::formats::SymmetricalJson;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};


type Recipients = Vec<usize>;
fn single_recipient(player_number: usize) -> Recipients {
    vec![player_number]
}

fn everyone_but(player_count: usize, player_number: usize) -> Recipients {
    let mut v = everyone(player_count);
    v.remove(player_number);

    v
}

fn everyone(player_count: usize) -> Recipients {
    (0..player_count).collect::<Vec<usize>>()
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // Bind a server socket
    let listener = TcpListener::bind("localhost:31194").await?;
    let (tx, _rx) = broadcast::channel::<(Value, Recipients)>(10);

    let data = Arc::new(Mutex::new(Game::new()));
    let mut player_count = 0;

    loop {
        let (socket, _addr) = listener.accept().await?;

        if player_count > 5 {
            println!("Too many players already! Ignoring new connection");
            continue;
        }

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        let player_number = player_count;
        let player = Player::new_with_default_deck(player_number);
        println!("Player #{} joined with UUID: {}", &player.player_number, &player.uuid);
        player_count += 1;

        let data = Arc::clone(&data);
        {
            let mut game = data.lock().unwrap();
            game.add_player(player);
        }

        // Duplicate the socket: one for serializing and one for deserializing
        let socket = socket.into_std()?;
        let socket2 = socket.try_clone()?;
        let socket = TcpStream::from_std(socket)?;
        let socket2 = TcpStream::from_std(socket2)?;

        let length_delimited =
            FramedRead::new(socket, LengthDelimitedCodec::new());
        let mut client_message_receiver: ClientMessageReceiver =
            tokio_serde::SymmetricallyFramed::new(
                length_delimited,
                SymmetricalJson::<ClientMessage>::default(),
            );

        let length_delimited =
            FramedWrite::new(socket2, LengthDelimitedCodec::new());
        let mut value_sender: ValueSender =
            tokio_serde::SymmetricallyFramed::new(
                length_delimited,
                SymmetricalJson::default()
            );

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle messages received from the broadcaster and pass them on
                    result = rx.recv() => {
                        let (value, recipients) = result.unwrap();

                        if recipients.contains(&player_number) {
                            value_sender.send(value).await.unwrap();
                        }
                    }

                    // Messages received from the client
                    result = client_message_receiver.try_next() => {
                        if let Some(msg) = result.unwrap() {
                            match msg {
                                ClientMessage::Ping => {
                                    println!("Got a ping from player {}!", player_number);
                                    value_sender.send(serde_json::to_value(ServerMessage::PingResponse).unwrap()).await.unwrap();
                                }
                                ClientMessage::ChatMessage{ message } => {
                                    let game = data.lock().unwrap();
                                    let player_count = game.player_count();
                                    // let sender = &game.players[player_number];
                                    // let author = sender.uuid;
                                    let message = serde_json::to_value(ServerMessage::ChatMessage{ author: player_number, message }).unwrap();
                                    let recipients = everyone_but(player_count, player_number);
                                    tx.send((message, recipients)).unwrap();
                                }
                                ClientMessage::StartGame { supply_list } => {
                                    let mut game = data.lock().unwrap();
                                    if game.started {
                                        let recipients = single_recipient(player_number);
                                        let message = serde_json::to_value(ServerMessage::GameAlreadyStarted).unwrap();
                                        tx.send((message, recipients)).unwrap();
                                        continue;
                                    }

                                    match game.generate_supply(supply_list) {
                                        Ok(()) => {
                                            game.started = true;
                                            for i in 0..game.player_count() {
                                                let recipients = single_recipient(i);
                                                let state = game.partial_game(i);
                                                let message = serde_json::to_value(ServerMessage::StartingGame { state }).unwrap();
                                                tx.send((message, recipients)).unwrap();
                                            }
                                        }
                                        Err(NotEnoughPlayers) => {
                                            let recipients = single_recipient(player_number);
                                            let message = serde_json::to_value(ServerMessage::NotEnoughPlayers).unwrap();
                                            tx.send((message, recipients)).unwrap();
                                        }
                                        _ => {
                                            panic!("Unknown error while starting!")
                                        }
                                    }
                                }
                                ClientMessage::PlayCard { index } => {
                                    let data = data.clone();
                                    // TODO: play the card
                                    play_card(data, player_number, index, &mut value_sender).await;
                                }

                                _ => {
                                    println!("Server received an unknown message from the client!");
                                    println!("Message: {:?}", msg);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
