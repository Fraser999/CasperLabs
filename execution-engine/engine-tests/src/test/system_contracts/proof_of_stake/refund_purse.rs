use engine_test_support::{
    internal::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_PAYMENT,
        DEFAULT_RUN_GENESIS_REQUEST,
    },
    DEFAULT_ACCOUNT_ADDR,
};
use types::{account::AccountHash, U512};

const CONTRACT_TRANSFER_PURSE_TO_ACCOUNT: &str = "transfer_purse_to_account.wasm";
const ACCOUNT_1_ADDR: AccountHash = AccountHash::new([1u8; 32]);

#[ignore]
#[test]
fn should_run_pos_refund_purse_contract_default_account() {
    let mut builder = initialize();
    refund_tests(&mut builder, DEFAULT_ACCOUNT_ADDR);
}

#[ignore]
#[test]
fn should_run_pos_refund_purse_contract_account_1() {
    let mut builder = initialize();
    transfer(&mut builder, ACCOUNT_1_ADDR, *DEFAULT_PAYMENT * 2);
    refund_tests(&mut builder, ACCOUNT_1_ADDR);
}

fn initialize() -> InMemoryWasmTestBuilder {
    let mut builder = InMemoryWasmTestBuilder::default();

    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    builder
}

fn transfer(builder: &mut InMemoryWasmTestBuilder, account_hash: AccountHash, amount: U512) {
    let exec_request = {
        ExecuteRequestBuilder::standard(
            DEFAULT_ACCOUNT_ADDR,
            CONTRACT_TRANSFER_PURSE_TO_ACCOUNT,
            (account_hash, amount),
        )
        .build()
    };

    builder.exec(exec_request).expect_success().commit();
}

fn refund_tests(builder: &mut InMemoryWasmTestBuilder, account_hash: AccountHash) {
    let exec_request = {
        let deploy = DeployItemBuilder::new()
            .with_address(account_hash)
            .with_deploy_hash([2; 32])
            .with_session_code("do_nothing.wasm", ())
            .with_payment_code("pos_refund_purse.wasm", (*DEFAULT_PAYMENT,))
            .with_authorization_keys(&[account_hash])
            .build();

        ExecuteRequestBuilder::new().push_deploy(deploy).build()
    };

    builder.exec(exec_request).expect_success().commit();
}
