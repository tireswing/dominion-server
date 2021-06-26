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
    let listener = TcpListener::bind("localhost:1234").await?;

    let data = Arc::new(Mutex::new(Game::new()));
    let mut player_count = 0;

    let (bridge_sender, _rx) = broadcast::channel::<(Value, Recipients)>(10);
    let bridge = ServerBridge {
        sender: bridge_sender.clone(),
    };
    let callbacks: Box<dyn Callbacks> = Box::new(bridge);

    let (broadcast_sender, _rx) = broadcast::channel::<(Value, Recipients)>(10);
    let join_handles = Arc::new(Mutex::new(Vec::new()));
    let task_handles = join_handles.clone();

    tokio::spawn(async move {
        loop {
            // let join_handles = join_handles.clone();
            let (socket, _addr) = listener.accept().await.unwrap();

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
            let socket = socket.into_std().unwrap();
            let socket2 = socket.try_clone().unwrap();
            let socket = TcpStream::from_std(socket).unwrap();
            let socket2 = TcpStream::from_std(socket2).unwrap();
    
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

            // Collect message channels into struct
            let mut message_channels = ServerMessageChannels {
                broadcast_sender,
                broadcast_receiver,
                value_sender,
                client_message_receiver
            };
    
            let callbacks = callbacks.clone();
    
            let handle =  tokio::spawn(async move {
                loop {
                    tokio::select! {
                        // Handle messages received from the broadcaster and pass them on
                        result = message_channels.broadcast_receiver.recv() => {
                            let (value, recipients) = result.unwrap();
    
                            let should_send = match recipients {
                                Recipients::Everyone => true,
                                Recipients::SingleRecipient { recipient } => recipient == player_number,
                                Recipients::MultipleRecipients { recipients } => recipients.contains(&player_number),
                            };
    
                            if should_send {
                                message_channels.value_sender.send(value).await.unwrap();
                            }
                        }
    
                        // Messages received from the client
                        result = message_channels.client_message_receiver.try_next() => {
                            if let Some(msg) = result.unwrap() {
                                let data = data.clone();
                                handle_client_message(msg, data, player_number, &callbacks, &mut message_channels).await;
                            }
                        }
                    }
                }
            });
            task_handles.lock().unwrap().push(handle);
        }
    });

    std::thread::sleep(std::time::Duration::new(60, 0));
    println!("{:?}", join_handles);

    loop {

    }
}
