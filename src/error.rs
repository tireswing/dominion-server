use thiserror::Error;

pub type DominionServerResult<T> = Result<T, DominionServerError>;

#[derive(Clone, Debug, Error)]
pub enum DominionServerError {
    #[error("")]
    BadReturn,
}
