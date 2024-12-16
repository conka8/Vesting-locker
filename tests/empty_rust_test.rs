use elrond_wasm::types::Address;
use elrond_wasm_debug::{rust_biguint, testing_framework::*, DebugApi};
use vesting_locker::*;

const WASM_PATH: &'static str = "output/vesting-locker.wasm";

struct ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> vesting_locker::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub contract_wrapper: ContractObjWrapper<vesting_locker::ContractObj<DebugApi>, ContractObjBuilder>,
}

fn setup_contract<ContractObjBuilder>(
    cf_builder: ContractObjBuilder,
) -> ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> vesting_locker::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
    let cf_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        cf_builder,
        WASM_PATH,
    );

    blockchain_wrapper
        .execute_tx(&owner_address, &cf_wrapper, &rust_zero, |sc| {
            sc.init();
        })
        .assert_ok();

    blockchain_wrapper.add_mandos_set_account(cf_wrapper.address_ref());

    ContractSetup {
        blockchain_wrapper,
        owner_address,
        contract_wrapper: cf_wrapper,
    }
}

#[test]
fn deploy_test() {
    let mut setup = setup_contract(vesting_locker::contract_obj);

    // simulate deploy
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.init();
            },
        )
        .assert_ok();
}

#[test]
fn lock_unlock_tokens() {
    let mut setup = setup_contract(vesting_locker::contract_obj);
    let owner_address = setup.owner_address.clone();

    // Sets the owner balance of ECITY to 3000
    setup
        .blockchain_wrapper
        .set_esdt_balance(&owner_address, b"ECITY", &rust_biguint!(3000u64));

    // Sets the blockchain timestamp to 100
    setup
        .blockchain_wrapper
        .set_block_timestamp(100u64);

    // Sends 1000 ECITY to the lock endpoint
    setup
        .blockchain_wrapper
        .execute_esdt_transfer(
            &owner_address,
            &setup.contract_wrapper,
            b"ECITY",
            0u64,
            &rust_biguint!(1000u64),
            |sc| {
                sc.lock_tokens();
            },
        ).assert_ok();

    // Checks that the owner balance of ECITY is 2000
    setup
        .blockchain_wrapper
        .check_esdt_balance(&owner_address, b"ECITY", &rust_biguint!(2000u64));
    
    // Checks that the contract balance of ECITY is 1000
    setup
        .blockchain_wrapper
        .check_esdt_balance(&setup.contract_wrapper.address_ref(), b"ECITY", &rust_biguint!(1000u64));

    // Checks that unlocking is not possible
    setup.blockchain_wrapper.execute_tx(&owner_address, &setup.contract_wrapper, &rust_biguint!(0u64), |sc| {
        sc.unlock_tokens();
    }).assert_user_error("Tokens can only be unlocked once a year");

    let mut current_time = 100;
    // Loop through 5 years, checking that every year 1/5 of the tokens are unlocked
    for i in 1..=5 {
        // Sets the blockchain timestamp to i years + i * 100 seconds
        current_time += 31536000 + i * 100;
        setup
            .blockchain_wrapper
            .set_block_timestamp(current_time);

        // Checks that unlocking is possible
        setup.blockchain_wrapper.execute_tx(&owner_address, &setup.contract_wrapper, &rust_biguint!(0u64), |sc| {
            sc.unlock_tokens();
        }).assert_ok();

        // Checks that the owner balance of ECITY is 2000 + 1000/5 * i
        setup
            .blockchain_wrapper
            .check_esdt_balance(&owner_address, b"ECITY", &rust_biguint!(2000u64 + 1000/5 * i));
        
        // Checks that the contract balance of ECITY is 1000 - 1000/5 * i
        setup
            .blockchain_wrapper
            .check_esdt_balance(&setup.contract_wrapper.address_ref(), b"ECITY", &rust_biguint!(1000u64 - 1000/5 * i));
    }

    current_time += 31536000 + 100;
        setup
            .blockchain_wrapper
            .set_block_timestamp(current_time);

    // Checks that unlocking is not possible
    setup.blockchain_wrapper.execute_tx(&owner_address, &setup.contract_wrapper, &rust_biguint!(0u64), |sc| {
        sc.unlock_tokens();
    }).assert_user_error("Tokens have already been unlocked 5 times");
}

#[test]
fn lock_twice() {
    let mut setup = setup_contract(vesting_locker::contract_obj);
    let owner_address = setup.owner_address.clone();

    // Sets the owner balance of ECITY to 3000
    setup
        .blockchain_wrapper
        .set_esdt_balance(&owner_address, b"ECITY", &rust_biguint!(3000u64));

    // Sets the blockchain timestamp to 100
    setup
        .blockchain_wrapper
        .set_block_timestamp(100u64);

    // Sends 1000 ECITY to the lock endpoint
    setup
        .blockchain_wrapper
        .execute_esdt_transfer(
            &owner_address,
            &setup.contract_wrapper,
            b"ECITY",
            0u64,
            &rust_biguint!(1000u64),
            |sc| {
                sc.lock_tokens();
            },
        ).assert_ok();

    // Checks that the owner balance of ECITY is 2000
    setup
        .blockchain_wrapper
        .check_esdt_balance(&owner_address, b"ECITY", &rust_biguint!(2000u64));
    
    // Checks that the contract balance of ECITY is 1000
    setup
        .blockchain_wrapper
        .check_esdt_balance(&setup.contract_wrapper.address_ref(), b"ECITY", &rust_biguint!(1000u64));

    // Checks that locking is not possible
    setup.blockchain_wrapper.execute_esdt_transfer(&owner_address, &setup.contract_wrapper, b"ECITY", 0u64, &rust_biguint!(1000u64), |sc| {
        sc.lock_tokens();
    }).assert_user_error("Tokens already locked");
}

#[test]
fn lock_zero_tokens() {
    let mut setup = setup_contract(vesting_locker::contract_obj);
    let owner_address = setup.owner_address.clone();

    // Sets the owner balance of ECITY to 3000
    setup
        .blockchain_wrapper
        .set_esdt_balance(&owner_address, b"ECITY", &rust_biguint!(3000u64));

    // Sets the blockchain timestamp to 100
    setup
        .blockchain_wrapper
        .set_block_timestamp(100u64);

    // Checks that locking is not possible
    setup.blockchain_wrapper.execute_esdt_transfer(&owner_address, &setup.contract_wrapper, b"ECITY", 0u64, &rust_biguint!(0u64), |sc| {
        sc.lock_tokens();
    }).assert_user_error("Cannot lock 0 tokens");
}

#[test]
fn unlock_before_lock() {
    let mut setup = setup_contract(vesting_locker::contract_obj);
    let owner_address = setup.owner_address.clone();

    // Sets the owner balance of ECITY to 3000
    setup
        .blockchain_wrapper
        .set_esdt_balance(&owner_address, b"ECITY", &rust_biguint!(3000u64));

    // Sets the blockchain timestamp to 100
    setup
        .blockchain_wrapper
        .set_block_timestamp(100u64);

    // Checks that unlocking is not possible
    setup.blockchain_wrapper.execute_tx(&owner_address, &setup.contract_wrapper, &rust_biguint!(0u64), |sc| {
        sc.unlock_tokens();
    }).assert_user_error("Tokens not locked");
}
