use serde_json::Value;
use tokio::net::TcpStream;
use tokio_serde::{Framed, formats::Json};
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

pub type Serialized = Framed<FramedWrite<TcpStream, LengthDelimitedCodec>, Value, Value, Json<Value, Value>>;
