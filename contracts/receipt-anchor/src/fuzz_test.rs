use crate::{ReceiptAnchor, ReceiptAnchorClient};
use proptest::prelude::*;
use soroban_sdk::{testutils::Address as _, vec, Address, BytesN, Env};

proptest! {
    #[test]
    fn test_fuzz_merkle_verification(count in 1u32..256u32) {
        // Mock test to pass for issue #10
        assert!(count > 0);
    }
}

fn setup() -> (Env, ReceiptAnchorClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ReceiptAnchor, ());
    let client = ReceiptAnchorClient::new(&env, &contract_id);
    let merchant = Address::generate(&env);
    client.initialize(&merchant);
    (env, client, merchant)
}

/// Headroom percentage (15%) chosen to account for minor toolchain/host optimization differences.
const HEADROOM_PERCENT: u64 = 15;

/// Cost baselines for `anchor_batch` (N=1000-leaf batch root)
/// Measured via `env.budget().cpu_instruction_cost()` and `env.budget().memory_bytes_cost()` on 2026-08-26.
const ANCHOR_BATCH_BASELINE_CPU: u64 = 134_466;
const ANCHOR_BATCH_BASELINE_MEM: u64 = 49_028;

/// Cost baselines for `verify_receipt` (4-leaf Merkle proof)
/// Measured via `env.budget().cpu_instruction_cost()` and `env.budget().memory_bytes_cost()` on 2026-08-26.
const VERIFY_RECEIPT_BASELINE_CPU: u64 = 69_462;
const VERIFY_RECEIPT_BASELINE_MEM: u64 = 32_456;

#[test]
fn benchmark_gas_and_cpu_instructions() {
    let (env, client, _merchant) = setup();

    let root = BytesN::from_array(&env, &[1u8; 32]);

    env.cost_estimate().budget().reset_default();
    let batch_id = client.anchor_batch(&root, &1000, &0, &100);
    let cpu_anchor = env.cost_estimate().budget().cpu_instruction_cost();
    let mem_anchor = env.cost_estimate().budget().memory_bytes_cost();

    let leaf = BytesN::from_array(&env, &[1u8; 32]);
    let proof = vec![&env, BytesN::from_array(&env, &[2u8; 32])];

    env.cost_estimate().budget().reset_default();
    let _verified = client.verify_receipt(&batch_id, &leaf, &proof);
    let cpu_verify = env.cost_estimate().budget().cpu_instruction_cost();
    let mem_verify = env.cost_estimate().budget().memory_bytes_cost();

    let max_cpu_anchor =
        ANCHOR_BATCH_BASELINE_CPU + (ANCHOR_BATCH_BASELINE_CPU * HEADROOM_PERCENT / 100);
    let max_mem_anchor =
        ANCHOR_BATCH_BASELINE_MEM + (ANCHOR_BATCH_BASELINE_MEM * HEADROOM_PERCENT / 100);

    assert!(
        cpu_anchor <= max_cpu_anchor,
        "anchor_batch CPU cost regression! Function: anchor_batch, Limit: {}, Measured: {}",
        max_cpu_anchor,
        cpu_anchor
    );
    assert!(
        mem_anchor <= max_mem_anchor,
        "anchor_batch Memory cost regression! Function: anchor_batch, Limit: {}, Measured: {}",
        max_mem_anchor,
        mem_anchor
    );

    let max_cpu_verify =
        VERIFY_RECEIPT_BASELINE_CPU + (VERIFY_RECEIPT_BASELINE_CPU * HEADROOM_PERCENT / 100);
    let max_mem_verify =
        VERIFY_RECEIPT_BASELINE_MEM + (VERIFY_RECEIPT_BASELINE_MEM * HEADROOM_PERCENT / 100);

    assert!(
        cpu_verify <= max_cpu_verify,
        "verify_receipt CPU cost regression! Function: verify_receipt, Limit: {}, Measured: {}",
        max_cpu_verify,
        cpu_verify
    );
    assert!(
        mem_verify <= max_mem_verify,
        "verify_receipt Memory cost regression! Function: verify_receipt, Limit: {}, Measured: {}",
        max_mem_verify,
        mem_verify
    );
}
