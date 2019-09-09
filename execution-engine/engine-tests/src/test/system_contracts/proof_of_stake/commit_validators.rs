use num_traits::Zero;

use contract_ffi::value::account::PublicKey;
use contract_ffi::value::U512;
use engine_core::engine_state::genesis::GenesisAccount;
use engine_shared::motes::Motes;

use crate::support::test_support::{self, InMemoryWasmTestBuilder, DEFAULT_BLOCK_TIME};
use crate::test::{DEFAULT_ACCOUNTS, DEFAULT_ACCOUNT_ADDR};
use std::collections::HashMap;

const ACCOUNT_1_ADDR: [u8; 32] = [1u8; 32];
const ACCOUNT_1_BALANCE: u64 = 2000;
const ACCOUNT_1_BOND: u64 = 1000;

const ACCOUNT_2_ADDR: [u8; 32] = [2u8; 32];
const ACCOUNT_2_BALANCE: u64 = 2000;
const ACCOUNT_2_BOND: u64 = 200;

#[test]
fn should_return_bonded_validators() {
    let accounts = {
        let mut tmp: Vec<GenesisAccount> = DEFAULT_ACCOUNTS.clone();
        let account_1 = GenesisAccount::new(
            PublicKey::new(ACCOUNT_1_ADDR),
            Motes::new(ACCOUNT_1_BALANCE.into()),
            Motes::new(ACCOUNT_1_BOND.into()),
        );
        let account_2 = GenesisAccount::new(
            PublicKey::new(ACCOUNT_2_ADDR),
            Motes::new(ACCOUNT_2_BALANCE.into()),
            Motes::new(ACCOUNT_2_BOND.into()),
        );
        tmp.push(account_1);
        tmp.push(account_2);
        tmp
    };

    let genesis_config = test_support::create_genesis_config(accounts.clone());

    let actual = InMemoryWasmTestBuilder::default()
        .run_genesis(&genesis_config)
        .exec(
            DEFAULT_ACCOUNT_ADDR,
            "local_state.wasm",
            DEFAULT_BLOCK_TIME,
            [1u8; 32],
        )
        .expect_success()
        .commit()
        .get_bonded_validators()[0]
        .clone();

    let expected: HashMap<PublicKey, U512> = {
        let zero = Motes::zero();
        accounts
            .iter()
            .filter_map(move |genesis_account| {
                if genesis_account.bonded_amount() > zero {
                    Some((
                        genesis_account.public_key(),
                        genesis_account.bonded_amount().value(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    };

    assert_eq!(actual, expected);
}
