//! Parser for `BankInfo.pabgb` (new in 1.12). Game deser sub_101F77D28;
//! exchange struct sub_101F77B04 (BankExchangeData); interest list sub_101FB5004.
//! Key-resolving readers consume a 4-byte key on the wire (resolved to u16 at
//! runtime): bank_investment_item_info/bank_pass_item_info=ItemKey,
//! use_bank_condition=ConditionKey, bank_license_knowledge_info=KnowledgeKey.
use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    // BankExchangeData (sub_101F77B04): targetInventoryInfo(InventoryKey u32) +
    // targetItemInfo(ItemKey u32) + targetItemAmount(f64) = 16B.
    pub struct BankExchangeData {
        pub target_inventory_info: u16,
        pub target_item_info: u32,
        pub target_item_amount: u64,
    }
}

py_binary_struct! {
    // bank_interest_rate element (sub_101FB5004): two 8-byte values per entry (16B).
    // Modeled u64 (raw 8 bytes) not f64 — these hold large integers that lose
    // precision through the f64 JSON layer; u64 round-trips exactly.
    pub struct BankInterestRate {
        pub amount: u64,
        pub rate: u64,
    }
}

py_binary_struct! {
    pub struct BankInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub bank_step: u32,
        pub bank_refresh_game_time_day: u32,
        pub bank_investment_item_info: u32,
        pub exchange_item_data: BankExchangeData,
        pub use_bank_condition: u32,
        pub min_investment_amount: u64,
        pub max_amount_for_interest: u64,
        // 1.16.00: RESOLVED against the binary's own per-field error strings
        // ("BankInfo의 _<field>를 읽어들이는데 실패했다" — NattKh method). The real
        // 1.16 field list is 16 fields and reads:
        //   … _minInvestmentAmount, _maxAmountForInterest,
        //     _maxAmountForInterestResult, _bankPassItemInfo,
        //     _bankLicenseKnowledgeInfo, _bankName, _fixInterestRateOnInvestment,
        //     _bankInterestRate
        // i.e. the 1.13 guess (9 provisional scalar bytes AFTER
        // _bankLicenseKnowledgeInfo) was wrong: it is ONE u64
        // `_maxAmountForInterestResult` placed BEFORE _bankPassItemInfo (17 → 16
        // scalar bytes, the -1B that desynced 1.16 at 0x51).
        pub max_amount_for_interest_result: u64,
        pub bank_pass_item_info: u32,
        pub bank_license_knowledge_info: u32,
        // These four ARE the binary's `_bankName` — a LocalizableString
        // (u8 category + u64 index + CString), which is byte-identical to the
        // u32+u32+u8+CString split the 1.13 pass guessed. So the 1.13 model was
        // right and only the +8 above was missing. Verified on the single bank
        // record (key 16960): category@0x4C, index@0x4D..0x54, len(14)@0x55,
        // "72842645340992"@0x59.
        // Names kept as-is: parser field names are a MOD CONTRACT — renaming
        // silently skips any mod referencing them.
        pub bank_new_a_113: u32,
        pub bank_new_b_113: u32,
        pub bank_new_c_113: u8,
        pub bank_new_str_113: CString<'a>,
        pub fix_interest_rate_on_investment: u8,
        pub bank_interest_rate: CArray<BankInterestRate>,
    }
}
