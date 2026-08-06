sed -i 's/RefundNotFound = 9,/RefundNotFound = 9,\n    MetadataTooLong = 10,\n    AmountExceedsMax = 11,/' contracts/refund-vault/src/lib.rs
sed -i 's/IsPaused,/IsPaused,\n    Metadata,\n    RefundMax,\n    Admins,\n    Threshold,/' contracts/refund-vault/src/lib.rs
