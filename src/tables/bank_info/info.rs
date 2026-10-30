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
    // 2.04.00 wire order (Mac reader sub_10203E674, names from its Korean asserts):
    // stringKey, key, isBlocked, fixInterestRateOnInvestment, bankPassItemInfo,
    // bankLicenseKnowledgeInfo, activeBankCondition, bankInvestmentItemInfo, bankStep,
    // bankRefreshGameTimeDay, exchangeItemData, minInvestmentAmount, maxAmountForInterest,
    // maxAmountForInterestResult, bankName, bankInterestRate. Same fields as 2.03,
    // re-sequenced. Field names are a MOD CONTRACT (DMM's Bank Max Return preset writes
    // `bank_interest_rate[*].amount`), so they stay; game names in comments.
    pub struct BankInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u16,
        pub is_blocked: u8,
        pub fix_interest_rate_on_investment: u8,
        pub bank_pass_item_info: u32,
        pub bank_license_knowledge_info: u32,
        pub use_bank_condition: u32,            // _activeBankCondition
        pub bank_investment_item_info: u32,
        pub bank_step: u32,
        pub bank_refresh_game_time_day: u32,
        pub exchange_item_data: BankExchangeData,
        pub min_investment_amount: u64,
        pub max_amount_for_interest: u64,
        pub max_amount_for_interest_result: u64,
        // `_bankName`, a LocalizableString (u8 category + u64 index + CString). The 1.13
        // pass split it as u32 + u32 + u8 + CString, byte-identical in total; the names
        // are kept because they are the mod contract.
        pub bank_new_a_113: u32,
        pub bank_new_b_113: u32,
        pub bank_new_c_113: u8,
        pub bank_new_str_113: CString<'a>,
        pub bank_interest_rate: CArray<BankInterestRate>,
    }
}

impl<'a> BankInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> std::io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        if *offset - start != entry_size {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!(
                "BankInfo: consumed {} of {} bytes", *offset - start, entry_size)));
        }
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("bankinfo.pabgb") }

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let mut out = Vec::with_capacity(data.len());
        for (i, (k, s, e)) in entry_ranges(&entries, data.len()).iter().enumerate() {
            let mut c = *s;
            let it = BankInfo::read_with_size(&data, &mut c, e - s)
                .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er));
            assert!(!it.bank_interest_rate.items.is_empty(), "e{}: a bank with no interest ladder", i);
            it.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "bankinfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        for (i, (key, start, end)) in entry_ranges(&entries, data.len()).iter().enumerate() {
            let mut cursor = *start;
            let item = BankInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            BankInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(from_json, from_typed, "entry {} key=0x{:x}: JSON round-trip diverges", i, key);
        }
    }
}
