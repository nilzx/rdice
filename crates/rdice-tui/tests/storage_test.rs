use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rdice_core::DiceEngine;
use rdice_core::die::{CUSTOM_PREFIX, FaceValue};

#[path = "../src/storage.rs"]
mod storage;

fn unique_path(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("rdice-tui-{label}-{stamp}.toml"))
}

#[test]
fn tui_storage_saves_and_loads_trays() {
    let path = unique_path("state");

    let mut engine = DiceEngine::new();
    engine
        .create_die(
            "Coin",
            vec![
                FaceValue::Text("heads".into()),
                FaceValue::Text("tails".into()),
            ],
        )
        .unwrap();
    engine.create_tray("test_tray").unwrap();
    engine.add_die_to_tray("D20", "test_tray").unwrap();
    engine
        .add_die_to_tray(&format!("{CUSTOM_PREFIX}Coin"), "test_tray")
        .unwrap();
    engine.roll_tray("test_tray").unwrap();
    engine.lock_slot("test_tray", 1).unwrap();

    storage::save(&path, &engine).unwrap();
    assert!(path.exists());

    let loaded = storage::load(&path).unwrap();
    assert_eq!(loaded.list_dice().len(), engine.list_dice().len());
    assert_eq!(loaded.list_trays().len(), 1);
    let tray = &loaded.list_trays()[0];
    assert_eq!(tray.slots.len(), 2);
    assert!(tray.slots[0].locked);
    assert_eq!(tray.next_slot_id, 3);

    let _ = std::fs::remove_file(path);
}
