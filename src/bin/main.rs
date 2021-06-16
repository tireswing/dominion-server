use dominion::prelude::*;
use dominion_server::prelude::*;

use std::sync::{Arc, Mutex};

use anyhow::Result;
use serde_json::Value;
use tokio::{net::{TcpListener, TcpStream}, sync::broadcast};
use tokio_serde::formats::SymmetricalJson;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};


#[tokio::main]
pub async fn main() -> Result<()> {
    // Bind a server socket
    let listener = TcpListener::bind("localhost:31194").await?;
    let (broadcast_sender, _rx) = broadcast::channel::<(Value, Recipients)>(10);

    let data = Arc::new(Mutex::new(Game::new()));
    let mut player_count = 0;

    loop {
        let (socket, _addr) = listener.accept().await?;

        if player_count > 5 {
            println!("Too many players already! Ignoring new connection");
            continue;
        }

        let broadcast_sender = broadcast_sender.clone();
        let broadcast_receiver = broadcast_sender.subscribe();

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
        let client_message_receiver: ClientMessageReceiver =
            tokio_serde::SymmetricallyFramed::new(
                length_delimited,
                SymmetricalJson::<ClientMessage>::default(),
            );

        let length_delimited =
            FramedWrite::new(socket2, LengthDelimitedCodec::new());
        let value_sender: ValueSender =
            tokio_serde::SymmetricallyFramed::new(
                length_delimited,
                SymmetricalJson::default()
            );

        let mut message_channels = ServerMessageChannels {
            broadcast_sender,
            broadcast_receiver,
            value_sender,
            client_message_receiver
        };

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle messages received from the broadcaster and pass them on
                    result = message_channels.broadcast_receiver.recv() => {
                        let (value, recipients) = result.unwrap();

                        if recipients.contains(&player_number) {
                            message_channels.value_sender.send(value).await.unwrap();
                        }
                    }

                    // Messages received from the client
                    result = message_channels.client_message_receiver.try_next() => {
                        if let Some(msg) = result.unwrap() {
                            let data = data.clone();
                            handle_client_message(msg, data, player_number, &mut message_channels).await;
                        }
                    }
                }
            }
        });
    }
}
