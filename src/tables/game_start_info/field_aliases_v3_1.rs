// v3.1 canonical (_camelCase) aliases for GameStartInfo. Names confirmed from
// the game deserializer error strings (sub_101F99838).

pub const FIELD_ALIASES_V3_1: &[(&str, &str)] = &[
    ("key", "_key"),
    ("string_key", "_stringKey"),
    ("is_blocked", "_isBlocked"),
    ("name", "_name"),
    ("field_info", "_fieldInfo"),
    ("start_quest_info", "_startQuestInfo"),
    ("use_instance_save_slot", "_useInstanceSaveSlot"),
    ("character_spawn_data_map", "_characterSpawnDataMap"),
];
