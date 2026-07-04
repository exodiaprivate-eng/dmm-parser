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
        pub bank_pass_item_info: u32,
        pub bank_license_knowledge_info: u32,
        // 1.13.00: new field block inserted here (+27 B once per record).
        // Byte-diff decisive (single bank record 16960): 9 fixed scalar bytes
        // followed by a CString (len-prefixed; "72842645340992" = 14 chars) then
        // the old fix_interest_rate/interest-rate list resume verbatim. Field
        // split within the 9 scalar bytes is roundtrip-equivalent; semantics
        // unconfirmed (a u32 read the low bytes 0x340 as a bogus CArray count =
        // the pre-fix V3 desync). Names are provisional.
        pub bank_new_a_113: u32,
        pub bank_new_b_113: u32,
        pub bank_new_c_113: u8,
        pub bank_new_str_113: CString<'a>,
        pub fix_interest_rate_on_investment: u8,
        pub bank_interest_rate: CArray<BankInterestRate>,
    }
}
