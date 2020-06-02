use num_derive::{FromPrimitive, ToPrimitive};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[repr(u8)]
#[derive(
    PartialEq, Eq, Copy, Clone, Debug, FromPrimitive, ToPrimitive, Serialize_repr, Deserialize_repr,
)]
pub enum Player {
    X = 0,
    O = 1,
}

impl Player {
    pub fn other(self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}
