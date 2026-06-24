// Field names recovered from IDA (NPCActivityInfo reader sub_101915238).
// Maps our snake_case field names to the engine's `_camelCase` identifiers.

pub const FIELD_ALIASES_V3_1: &[(&str, &str)] = &[
    ("key", "_key"),
    ("string_key", "_stringKey"),
    ("is_blocked", "_isBlocked"),
    ("activity_tag_list", "_activityTagList"),
    ("sequence_list", "_sequenceList"),
    ("interaction_task_list", "_interactionTaskList"),
    ("group_task_list", "_groupTaskList"),
    ("catch_action_task_list", "_catchActionTaskList"),
    ("flow_control_list", "_flowControlList"),
];
