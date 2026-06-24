//! IDA-derived parser for `npcactivityinfo.pabgb`.
//!
//! Reader: `sub_101915238` (CrimsonDesert_Steam 1.11, Mac IDB base 0x1_0000_0000).
//! Sequential table. Field order + wire sizes recovered via the Korean
//! per-field error strings (`NPCActivityInfo의 _<field>를 읽어들이는데 실패했다`)
//! and decompilation of each field reader.
//!
//! Wire layout (sub_101915238):
//!   1. _key                 u32         (sub_100FAAE34, 4B)
//!   2. _stringKey           CString     (sub_1006E817C)
//!   3. _isBlocked           u8          (sub_1006E7EEC, 1B)
//!   4. _activityTagList     CArray<u32> (sub_100CC2174)
//!   5. _sequenceList        CArray<NpcActivitySequence>        (sub_101949454)
//!   6. _interactionTaskList CArray<NpcActivityInteractionTask> (sub_1019495FC)
//!   7. _groupTaskList       CArray<NpcActivityGroupTask>       (sub_10194990C)
//!   8. _catchActionTaskList CArray<NpcActivityCatchActionTask> (sub_101949BFC)
//!   9. _flowControlList     CArray<NpcActivityFlowControl>     (sub_101949EE8)
//!
//! Inner element readers:
//!   NPCActivitySequence (sub_101949204): CArray of
//!     NPCActivityInstructionLocator { u8 instruction_type, u8 npc_activity_type,
//!                                     u32 index }.
//!   NPCActivityTask base (sub_101914960): u8 instruction_type, u8 activity_type,
//!     f32 duration_seconds (sub_1006E800C, 4B), u32 end_condition_info
//!     (sub_100F840EC ConditionKey, 4B wire → u16 hash at runtime).
//!   NPCActivityInteractionTask (sub_101914B78): base + u32 self_condition_info +
//!     u32 target_condition_info + u8 interaction_type (sub_101409844) +
//!     u32 interaction_key (sub_100F9C5C4) + CString interaction_tag
//!     (sub_1006E8320 — reads u32 len then advances the stream by len).
//!   NPCActivityGroupTask (sub_101914DD0): base + u32 self_condition_info +
//!     u32 target_condition_info + u8 group_activity_type + u32 group_limit.
//!   NPCActivityCatchActionTask (sub_101915018): base + u32 self_condition_info +
//!     u32 catch_target_condition + u32 catch_action_hash.
//!   NPCActivityFlowControl (sub_101914838): u8 instruction_type, u8 control_type,
//!     CArray<u32> sequence_index_list (sub_100CC2174).
//!
//! Note: end/self/target condition fields and interaction_key are stored wire as
//! u32 (a ConditionKey / hash). The engine resolves them to a u16 runtime index;
//! the parser keeps the raw u32 so the table round-trips byte-exact.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    /// `_instructionLocatorList` element (NPCActivityInstructionLocator).
    pub struct NpcActivityInstructionLocator {
        pub instruction_type: u8,
        pub npc_activity_type: u8,
        pub index: u32,
    }
}

py_binary_struct! {
    /// `_sequenceList` element (NPCActivitySequence). Body is just the
    /// instruction-locator list.
    pub struct NpcActivitySequence {
        pub instruction_locator_list: CArray<NpcActivityInstructionLocator>,
    }
}

py_binary_struct! {
    /// `_interactionTaskList` element (NPCActivityInteractionTask).
    /// First four fields are the inlined NPCActivityTask base class.
    pub struct NpcActivityInteractionTask<'a> {
        pub instruction_type: u8,
        pub activity_type: u8,
        pub duration_seconds: f32,
        pub end_condition_info: u32,
        pub self_condition_info: u32,
        pub target_condition_info: u32,
        pub interaction_type: u8,
        pub interaction_key: u32,
        pub interaction_tag: CString<'a>,
    }
}

py_binary_struct! {
    /// `_groupTaskList` element (NPCActivityGroupTask).
    /// First four fields are the inlined NPCActivityTask base class.
    pub struct NpcActivityGroupTask {
        pub instruction_type: u8,
        pub activity_type: u8,
        pub duration_seconds: f32,
        pub end_condition_info: u32,
        pub self_condition_info: u32,
        pub target_condition_info: u32,
        pub group_activity_type: u8,
        pub group_limit: u32,
    }
}

py_binary_struct! {
    /// `_catchActionTaskList` element (NPCActivityCatchActionTask).
    /// First four fields are the inlined NPCActivityTask base class.
    pub struct NpcActivityCatchActionTask {
        pub instruction_type: u8,
        pub activity_type: u8,
        pub duration_seconds: f32,
        pub end_condition_info: u32,
        pub self_condition_info: u32,
        pub catch_target_condition: u32,
        pub catch_action_hash: u32,
    }
}

py_binary_struct! {
    /// `_flowControlList` element (NPCActivityFlowControl). Base class is
    /// NPCActivityInstruction (just `instruction_type`).
    pub struct NpcActivityFlowControl {
        pub instruction_type: u8,
        pub control_type: u8,
        pub sequence_index_list: CArray<u32>,
    }
}

py_binary_struct! {
    pub struct NpcActivityInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub activity_tag_list: CArray<u32>,
        pub sequence_list: CArray<NpcActivitySequence>,
        pub interaction_task_list: CArray<NpcActivityInteractionTask<'a>>,
        pub group_task_list: CArray<NpcActivityGroupTask>,
        pub catch_action_task_list: CArray<NpcActivityCatchActionTask>,
        pub flow_control_list: CArray<NpcActivityFlowControl>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf {
        crate::testenv::resolve("npcactivityinfo.pabgb")
    }

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };

        // Sequential read to EOF (matches the `s!` dispatch path).
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(NpcActivityInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len());

        // Cross-check each record ends exactly on its pabgh boundary.
        if let Some(entries) =
            load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy())
        {
            let ranges = entry_ranges(&entries, data.len());
            for (i, (key, start, end)) in ranges.iter().enumerate() {
                let mut cursor = *start;
                NpcActivityInfo::read_from(&data, &mut cursor).unwrap();
                assert_eq!(
                    cursor, *end,
                    "entry {} key=0x{:x}: under/over-read at boundary",
                    i, key
                );
            }
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "npcactivityinfo roundtrip bytes mismatch");
    }
}
