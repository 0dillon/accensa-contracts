#![cfg(test)]

use crate::{Error, RefundVault, RefundVaultClient};
use proptest::prelude::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, BytesN, Env,
};

const FLOAT: i128 = 1_000_000_000_000;

fn setup(window: u32) -> (Env, RefundVaultClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let merchant = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(token_admin);
    let token = sac.address();
    StellarAssetClient::new(&env, &token).mint(&merchant, &FLOAT);

    let contract_id = env.register(RefundVault, ());
    let client = RefundVaultClient::new(&env, &contract_id);
    client.initialize(&merchant, &token, &window);

    (env, client, merchant, token)
}

proptest! {
    #[test]
    fn test_fuzz_deposit_extreme_amounts(amount in proptest::num::i128::ANY) {
        let (_, client, merchant, _) = setup(100);
        let res = client.try_deposit(&merchant, &amount);
        if amount <= 0 {
            assert_eq!(res, Err(Ok(Error::InvalidAmount)));
        } else if amount > FLOAT {
            // It will panic in the token contract
            assert!(res.is_err());
        } else {
            assert!(res.is_ok());
        }
    }

    #[test]
    fn test_fuzz_ttl_extension(ledger in 1u32..1000000u32) {
        let (env, client, _, _) = setup(100);
        env.ledger().set_sequence_number(ledger);

        let payment_ref = BytesN::from_array(&env, &[0; 32]);
        let res = client.try_extend_refund_ttl(&payment_ref);
        assert_eq!(res, Err(Ok(Error::RefundNotFound)));
    }

    /// Invariant Test (#94): RefundVault's total internal token balance MUST equal
    /// sum of all recorded individual user claims/liabilities (Total Deposits - Total Refunds - Total Withdrawals).
    #[test]
    fn test_fuzz_refund_vault_balance_invariant(
        deposit_amounts in proptest::collection::vec(1i128..10_000_000i128, 1..5),
        refund_amounts in proptest::collection::vec(1i128..1_000_000i128, 1..5),
        withdraw_amounts in proptest::collection::vec(1i128..1_000_000i128, 1..5)
    ) {
        let (env, client, merchant, token) = setup(100);
        let token_client = soroban_sdk::token::Client::new(&env, &token);

        let mut expected_balance: i128 = 0;

        // Deposits
        for amt in deposit_amounts {
            if client.try_deposit(&merchant, &amt).is_ok() {
                expected_balance += amt;
            }
            let actual_balance = token_client.balance(&client.address);
            assert_eq!(actual_balance, expected_balance, "Invariant mismatch after deposit");
        }

        // Refunds
        for (idx, amt) in refund_amounts.into_iter().enumerate() {
            let mut ref_bytes = [0u8; 32];
            ref_bytes[0] = (idx + 1) as u8;
            let payment_ref = BytesN::from_array(&env, &ref_bytes);
            let recipient = Address::generate(&env);

            if client.try_refund(&payment_ref, &recipient, &amt, &0).is_ok() {
                expected_balance -= amt;
            }
            let actual_balance = token_client.balance(&client.address);
            assert_eq!(actual_balance, expected_balance, "Invariant mismatch after refund");
        }

        // Withdrawals
        for amt in withdraw_amounts {
            if client.try_withdraw(&amt, &merchant).is_ok() {
                expected_balance -= amt;
            }
            let actual_balance = token_client.balance(&client.address);
            assert_eq!(actual_balance, expected_balance, "Invariant mismatch after withdrawal");
        }
    }
}

/// Headroom percentage (15%) chosen to account for minor toolchain/host optimization differences.
const HEADROOM_PERCENT: u64 = 15;

/// Cost baselines for `RefundVault::refund`
/// Measured via `env.cost_estimate().budget().cpu_instruction_cost()` and `env.cost_estimate().budget().memory_bytes_cost()` on 2026-08-26.
const REFUND_BASELINE_CPU: u64 = 397_721;
const REFUND_BASELINE_MEM: u64 = 131_994;

#[test]
fn test_refund_resource_cost_budget() {
    let (env, client, merchant, _token) = setup(100);
    client.deposit(&merchant, &1_000_000);

    let payment_ref = BytesN::from_array(&env, &[1u8; 32]);
    let recipient = Address::generate(&env);

    env.cost_estimate().budget().reset_default();
    client.refund(&payment_ref, &recipient, &100_000, &0);
    let cpu_refund = env.cost_estimate().budget().cpu_instruction_cost();
    let mem_refund = env.cost_estimate().budget().memory_bytes_cost();

    let max_cpu_refund = REFUND_BASELINE_CPU + (REFUND_BASELINE_CPU * HEADROOM_PERCENT / 100);
    let max_mem_refund = REFUND_BASELINE_MEM + (REFUND_BASELINE_MEM * HEADROOM_PERCENT / 100);

    assert!(
        cpu_refund <= max_cpu_refund,
        "RefundVault::refund CPU cost regression! Function: refund, Limit: {}, Measured: {}",
        max_cpu_refund,
        cpu_refund
    );
    assert!(
        mem_refund <= max_mem_refund,
        "RefundVault::refund Memory cost regression! Function: refund, Limit: {}, Measured: {}",
        max_mem_refund,
        mem_refund
    );
}
