# Roadmap

## Issue #443: stACC liquid staking derivative minting

- [x] Implement stACC liquid staking derivative minting (#443)
  - New `liquid_staking` module in `contracts/governance/src/liquid_staking.rs`
  - 1:1 mint/burn of stACC against locked underlying tokens
  - Exchange rate progression and lock-epoch-based redemption
  - CHANGELOG.md entry added

## Issue #749: rustdoc for receipt-shard's 4 exported names

- [x] Document `DataKey`, `ReceiptShard`, `get_batch` and `extend_batch_ttl` (#749)
  - `contracts/receipt-shard/src/lib.rs` — doc comments only, no signature, body or test changes

## Issue #790: rustdoc for `BuybackConfiguredEvent`'s 5 public fields

- [x] Document the struct's 5 public fields (#790)
  - `contracts/treasury/src/buyback.rs` — doc comments only, field/type/order/derives unchanged
