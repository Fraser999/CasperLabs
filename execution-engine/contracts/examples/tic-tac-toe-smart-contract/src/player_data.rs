use alloc::vec::Vec;

use contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use num_traits::{FromPrimitive, ToPrimitive};

use serde::{Deserialize, Serialize};
use tic_tac_toe_logic::player::Player;
use types::{
    account::{PublicKey, ED25519_SERIALIZED_LENGTH},
    bytesrepr::{self, FromBytes, ToBytes, U8_SERIALIZED_LENGTH},
    CLType, CLTyped, URef, UREF_SERIALIZED_LENGTH,
};

use crate::error::Error;

const PLAYER_DATA_BYTES_SIZE: usize =
    U8_SERIALIZED_LENGTH + ED25519_SERIALIZED_LENGTH + UREF_SERIALIZED_LENGTH;

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

impl ToBytes for PlayerData {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = Vec::with_capacity(PLAYER_DATA_BYTES_SIZE);
        result.push(self.piece.to_u8().unwrap());
        result.append(&mut self.opponent.to_bytes()?);
        result.append(&mut self.status_key.to_bytes()?);

        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        PLAYER_DATA_BYTES_SIZE
    }
}

impl FromBytes for PlayerData {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (piece, remainder) = u8::from_bytes(bytes)?;
        let (opponent, remainder) = PublicKey::from_bytes(remainder)?;
        let (status_key, remainder) = URef::from_bytes(remainder)?;
        let piece = FromPrimitive::from_u8(piece).ok_or(bytesrepr::Error::Formatting)?;
        Ok((
            PlayerData {
                piece,
                opponent,
                status_key,
            },
            remainder,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::PlayerData;
    use types::{account::PublicKey, bytesrepr, AccessRights, URef};

    use tic_tac_toe_logic::player::Player;

    #[test]
    fn player_data_round_trip() {
        let player_data = PlayerData {
            piece: Player::X,
            opponent: PublicKey::ed25519_from([3u8; 32]),
            status_key: URef::new([5u8; 32], AccessRights::READ_ADD_WRITE),
        };
        bytesrepr::test_serialization_roundtrip(&player_data);
    }
}
