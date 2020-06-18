use contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};

use serde::{Deserialize, Serialize};
use tic_tac_toe_logic::player::Player;
use types::{account::PublicKey, CLType, CLTyped, URef};

use crate::error::Error;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    piece: Player,
    opponent: PublicKey,
    status_key: URef,
}

impl PlayerData {
    pub fn read_local(key: PublicKey) -> Option<PlayerData> {
        storage::read_local(&key).unwrap_or_revert_with(Error::PlayerDataDeserialization)
    }

    pub fn write_local(key: PublicKey, piece: Player, opponent: PublicKey, status_key: URef) {
        let data = PlayerData {
            piece,
            opponent,
            status_key,
        };

        storage::write_local(key, data);
    }

    pub fn piece(&self) -> Player {
        self.piece
    }

    pub fn opponent(&self) -> PublicKey {
        self.opponent
    }

    pub fn status_key(&self) -> URef {
        self.status_key
    }
}

impl CLTyped for PlayerData {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

#[cfg(test)]
mod tests {
    use super::PlayerData;
    use types::{account::PublicKey, encoding, AccessRights, URef};

    use tic_tac_toe_logic::player::Player;

    #[test]
    fn player_data_round_trip() {
        let player_data = PlayerData {
            piece: Player::X,
            opponent: PublicKey::ed25519_from([3u8; 32]),
            status_key: URef::new([5u8; 32], AccessRights::READ_ADD_WRITE),
        };
        encoding::test_serialization_roundtrip(&player_data);
    }
}
