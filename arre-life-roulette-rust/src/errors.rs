use thiserror::Error;
use std::error::Error;

pub type ArreResult<T> = Result<T, Box<dyn Error>>;

#[derive(Error, Debug)]
pub enum ArreError {
    // Godot errors
    #[error("[color=red]Gd object at [b]`{0}`[/b] is null[/color]")]
    NullGd(String),
    #[error("[color=red][b]`instantiate()`[/b] failed for [b]`{0}`[/b] at [b]`{1}`[/b][/color]")]
    InstantiateFailed(String, String),
    #[error("[color=red][b]`try_cast()`[/b] failed for [b]`{0}`[/b] at [b]`{1}`[/b][/color]")]
    CastFailed(String, String),
    // Core errors
    #[error("[color=red] Attempt to operate on non persisted item [/color]")]
    ItemNotPersisted(),
}
