use soroban_sdk::{Env, Address, xdr::{Asset, AssetCode4, AlphaNum4, AccountId, PublicKey, Uint256}};

pub fn test() {
    let env = Env::default();
    let issuer = Address::generate(&env);
    
    // How to create an Asset?
}
