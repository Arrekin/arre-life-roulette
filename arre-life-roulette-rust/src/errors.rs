use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArreError {
    #[error("Gd object at `{0}` is null")]
    NullGd(String),
}