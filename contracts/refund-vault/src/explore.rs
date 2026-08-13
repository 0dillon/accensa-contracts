use soroban_sdk::{Env, Address};

#[test]
fn explore_env() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin);
    // Let's see what other methods exist
}
