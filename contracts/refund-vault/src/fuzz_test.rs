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
}
