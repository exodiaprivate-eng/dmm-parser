//! GameCondition tree case 6: pa::ConditionGimmickData_CheckProperty.
//!
//! Hand-written from IDA decompile of sub_141CB6480.
//! Stream layout: [u32 property_id]

use crate::py_binary_struct;

py_binary_struct! {
    pub struct ConditionGimmickData {
        pub property_id: u32,
    }
}
