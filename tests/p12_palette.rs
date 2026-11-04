//! P12 palette/help/events tests (RED first).

#[test]
fn fuzzy_ranks_snapshot_first() {
    use virsh_tui::ui::overlays::palette::fuzzy_rank;
    let items = vec!["snapshot-create-as", "start", "shutdown", "net-start"];
    let ranked = fuzzy_rank("snap", &items);
    assert!(ranked.first().is_some_and(|s| s.contains("snapshot")));
}

#[test]
fn help_index_parses_all_fixtures() {
    let text = std::fs::read_to_string("tests/fixtures/virsh/help_index.txt").unwrap();
    let entries = virsh_tui::backend::virsh::parse_help::parse_help_index(&text);
    assert!(entries.len() > 200);
    for e in entries.iter().take(5) {
        assert!(!e.name.is_empty());
    }
}

#[test]
fn keymap_sections_cover_defaults() {
    use virsh_tui::ui::overlays::help::sections;
    let mut map = virsh_tui::input::keymap::default_map();
    let secs = sections(&map);
    assert!(secs.len() >= 8);
    assert!(secs.iter().any(|s| s.title == "Navigation"));
    // Remapped keys show up in the help.
    map.retain(|_, a| *a != virsh_tui::input::engine::KeyAction::Shutdown);
    map.insert(
        vec![String::from("Space"), String::from("q")],
        virsh_tui::input::engine::KeyAction::Shutdown,
    );
    let secs = sections(&map);
    let domain = secs.iter().find(|s| s.title == "Domain").unwrap();
    assert!(
        domain
            .rows
            .iter()
            .any(|(k, d, _)| k == "␣q" && d.starts_with("shutdown"))
    );
}

#[test]
fn single_source_of_truth_keymap_rebind_reflected_everywhere() {
    use virsh_tui::input::engine::{KeyAction, KeyEngine};
    use virsh_tui::input::key::KeySeq;
    use virsh_tui::ui::overlays::help::sections;
    use virsh_tui::ui::overlays::palette::curated_with_map;
    use virsh_tui::ui::widgets::whichkey::entries_for_map;

    let mut map = virsh_tui::input::keymap::default_map();

    // Rebind NewDomain from Space n to Space a
    map.retain(|_, a| *a != KeyAction::NewDomain);
    map.insert(
        vec![String::from("Space"), String::from("a")],
        KeyAction::NewDomain,
    );

    // 1. Behavior in KeyEngine
    let mut engine = KeyEngine::with_map(map.clone());
    assert!(engine.feed(&KeySeq::parse("Space")).is_empty());
    let resolved = engine.feed(&KeySeq::parse("a"));
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].action, KeyAction::NewDomain);

    // 2. Leader key pop-up (WhichKey)
    let wk_entries = entries_for_map(&map, "Space");
    assert!(wk_entries.iter().any(|(k, d)| k == "a" && d == "new domain"));
    assert!(!wk_entries.iter().any(|(k, _)| k == "n"));

    // 3. Help overlay
    let secs = sections(&map);
    let leader_sec = secs.iter().find(|s| s.title.starts_with("Leader")).unwrap();
    assert!(
        leader_sec
            .rows
            .iter()
            .any(|(k, d, _)| k == "␣a" && d == "new domain")
    );
    assert!(!leader_sec.rows.iter().any(|(k, _, _)| k == "␣n"));

    // Rebind Shutdown from S to Space q
    map.retain(|_, a| *a != KeyAction::Shutdown);
    map.insert(
        vec![String::from("Space"), String::from("q")],
        KeyAction::Shutdown,
    );

    // 4. WhichKey shows shutdown under Space
    let wk_entries = entries_for_map(&map, "Space");
    assert!(
        wk_entries
            .iter()
            .any(|(k, d)| k == "q" && d.starts_with("shutdown"))
    );

    // 5. Command palette shows ␣q for shutdown
    let pal_items = curated_with_map(&map);
    let shutdown_item = pal_items.iter().find(|i| i.title == "shutdown").unwrap();
    assert_eq!(shutdown_item.key, "␣q");
}

#[test]
fn events_filter() {
    use virsh_tui::ui::views::events::{EventFilter, EventsState};
    let mut st = EventsState::demo();
    assert_eq!(st.visible().len(), 4);
    st.filter = EventFilter::Lifecycle;
    assert!(!st.visible().is_empty());
}

#[test]
fn every_help_entry_parses() {
    use virsh_tui::backend::virsh::parse_help::{parse_help_cmd, parse_help_index};
    if std::process::Command::new("virsh")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let index_out = std::process::Command::new("virsh")
        .args(["help"])
        .env("LC_ALL", "C")
        .output()
        .expect("virsh help runs");
    let entries = parse_help_index(&String::from_utf8_lossy(&index_out.stdout));
    assert!(entries.len() > 200, "got {} entries", entries.len());
    for e in &entries {
        let out = std::process::Command::new("virsh")
            .args(["help", &e.name])
            .env("LC_ALL", "C")
            .output()
            .expect("virsh help <cmd> runs");
        assert!(out.status.success(), "help {} failed", e.name);
        let doc = parse_help_cmd(&String::from_utf8_lossy(&out.stdout));
        assert_eq!(doc.name, e.name, "name mismatch for {}", e.name);
    }
}
