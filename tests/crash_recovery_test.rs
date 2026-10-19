use iris_projects::{CrashJournal, Project};
use std::env;

#[test]
fn test_project_atomic_save_and_load() {
    let temp_dir = env::temp_dir();
    let project_file = temp_dir.join("iris_test_project.iris");

    let mut project = Project::new("prj_test_123", "Sunset Vacation");
    project.timeline.fps_num = 60;
    project.timeline.fps_den = 1;

    // Save atomically
    project
        .save_to_file(&project_file)
        .expect("Atomic save failed");

    // Load and verify
    let loaded = Project::load_from_file(&project_file).expect("Load failed");
    assert_eq!(loaded.id, "prj_test_123");
    assert_eq!(loaded.name, "Sunset Vacation");
    assert_eq!(loaded.timeline.fps_num, 60);

    let _ = std::fs::remove_file(&project_file);
}

#[test]
fn test_crash_journal_write_and_recovery() {
    let temp_dir = env::temp_dir();
    let project_file = temp_dir.join("iris_crash_sim.iris");

    let mut journal = CrashJournal::new(&project_file);
    let _ = journal.clear();

    assert!(!journal.has_unrecovered_crash());

    // Record commands in WAL
    journal
        .append_entry("SPLIT_CLIP", "{\"clip_id\": \"clip_1\", \"time\": 4.5}")
        .expect("WAL write failed");
    journal
        .append_entry("ADJUST_EXPOSURE", "{\"exposure\": 1.25}")
        .expect("WAL write failed");

    // Check crash detection
    assert!(journal.has_unrecovered_crash());

    let entries = journal.read_entries().expect("WAL read failed");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].action_type, "SPLIT_CLIP");
    assert_eq!(entries[1].action_type, "ADJUST_EXPOSURE");

    // Clean up
    journal.clear().expect("WAL clear failed");
    assert!(!journal.has_unrecovered_crash());
}
