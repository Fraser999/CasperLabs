use serde::Serialize;
use types::account::PublicKey;

mod big_array {
    use serde_big_array::big_array;

    big_array! { BigArray; }
}

#[derive(Serialize)]
pub struct StateKey(#[serde(with = "big_array::BigArray")] [u8; 64]);

impl StateKey {
    pub fn new(x_player: PublicKey, o_player: PublicKey) -> StateKey {
        let mut result = [0u8; 64];
        for (i, j) in x_player
            .as_bytes()
            .iter()
            .chain(o_player.as_bytes().iter())
            .enumerate()
        {
            result[i] = *j;
        }
        StateKey(result)
    }
}
