use crate::api::{ClientMessage, ServerMessage};

use serde_json::Value;
use tokio::{net::TcpStream, sync::broadcast::{Receiver, Sender}};
use tokio_serde::{Framed, formats::Json};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub type MessageReceiver<T> = Framed<FramedRead<TcpStream, LengthDelimitedCodec>, T, T, Json<T, T>>;
pub type MessageSender<T> = Framed<FramedWrite<TcpStream, LengthDelimitedCodec>, T, T, Json<T, T>>;

pub type ClientMessageReceiver = MessageReceiver<ClientMessage>;
pub type ServerMessageReceiver = MessageReceiver<ServerMessage>;
pub type ValueSender = MessageSender<Value>;

pub struct ServerMessageChannels {
    pub broadcast_sender: Sender<(Value, Vec<usize>)>,
    pub broadcast_receiver: Receiver<(Value, Vec<usize>)>,
    pub value_sender: ValueSender,
    pub client_message_receiver: ClientMessageReceiver,
}
