#![no_std]
#![no_main]

use contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use types::{contracts::DEFAULT_ENTRY_POINT_NAME, ApiError, Key};

#[no_mangle]
pub extern "C" fn call() {
    let contract_to_call: [u8; 32] = runtime::get_arg(0)
        .unwrap_or_revert_with(ApiError::MissingArgument)
        .unwrap_or_revert_with(ApiError::InvalidArgument);
    runtime::call_contract::<_, ()>(Key::Hash(contract_to_call), DEFAULT_ENTRY_POINT_NAME, ());
}
