use thiserror::Error;
use std::error::Error;

pub type BoxedError = Box<dyn Error>;
pub type ArreResult<T> = Result<T, BoxedError>;

#[derive(Error, Debug)]
pub enum ArreError {
    // Godot errors
    #[error("[color=red]Gd object at [b]`{0}`[/b] is null[/color]")]
    NullGd(String),
    #[error("[color=red][b]`instantiate()`[/b] failed for [b]`{0}`[/b] at [b]`{1}`[/b][/color]")]
    InstantiateFailed(String, String),
    #[error("[color=red][b]`try_cast()`[/b] failed for [b]`{0}`[/b] at [b]`{1}`[/b][/color]")]
    CastFailed(String, String),
    #[error("[color=red][b]`create_tween()`[/b] failed for [b]`{0}`[/b] at [b]`{1}`[/b][/color]")]
    CreateTweenFailed(String, String),
    #[error("[color=red]Unexpected None at [b]`{0}`[/b][/color]")]
    UnexpectedNone(String),
    // Logic errors
    #[error("[color=red]Set of selected items is empty[/color]")]
    ItemsSelectionIsEmpty(),
    #[error("[color=red]Owned bus cannot be cloned[/color]")]
    OwnedBusCannotBeCloned(),
    // Core errors
    #[error("[color=red] Attempt to operate on non persisted item [/color]")]
    ItemNotPersisted(),
}
