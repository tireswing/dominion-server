use dominion::game::prelude::*;

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientMessage {
    Ping,
    ChatMessage { message: String },
    StartGame { supply_list: CardList },
    PlayCard { index: usize },
    BuyCard { card: Box<dyn Card> },
    EndTurn,
    React { card: Box<dyn Card> },
    NoMoreReactions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerMessage {
    PingResponse,
    ChatMessage { author: usize, message: String },
    StartingGame { state: PartialGame },
    CurrentState { state: PartialGame },
    GameAlreadyStarted,
    NotEnoughPlayers,
    NotYourTurn,
    IllegalPlay { card: Box<dyn Card>, reason: IllegalPlayReason },
    NotEnoughCoins,
    SupplyPileEmpty,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum IllegalPlayReason {
    WrongPhase,
}
