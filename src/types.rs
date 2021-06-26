use crate::prelude::*;

use serde_json::Value;
use tokio::{net::TcpStream, sync::broadcast::{Receiver, Sender}};
use tokio_serde::{Framed, formats::Json};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub type MessageReceiver<T> = Framed<FramedRead<TcpStream, LengthDelimitedCodec>, T, T, Json<T, T>>;
pub type MessageSender<T> = Framed<FramedWrite<TcpStream, LengthDelimitedCodec>, T, T, Json<T, T>>;

pub type ClientMessageReceiver = MessageReceiver<ClientMessage>;
pub type ServerMessageReceiver = MessageReceiver<ServerMessage>;
pub type ValueSender = MessageSender<Value>;

pub type BroadcastSender = Sender<(Value, Recipients)>;
pub type BroadcastReceiver = Receiver<(Value, Recipients)>;

pub struct ServerMessageChannels {
    pub broadcast_sender: BroadcastSender,
    pub broadcast_receiver: BroadcastReceiver,
    pub value_sender: ValueSender,
    pub client_message_receiver: ClientMessageReceiver,
}
