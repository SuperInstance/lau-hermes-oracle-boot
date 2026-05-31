use lau_hermes_oracle_boot::*;
use tempfile::TempDir;

// ── BootPhase tests ──────────────────────────────────────────────────────────

#[test]
fn phase_all_returns_seven_phases() {
    assert_eq!(BootPhase::all().len(), 7);
}

#[test]
fn phase_order_is_correct() {
    assert!(BootPhase::Init.order() < BootPhase::LoadConfig.order());
    assert!(BootPhase::LoadConfig.order() < BootPhase::ConnectDB.order());
    assert!(BootPhase::ConnectDB.order() < BootPhase::LoadRooms.order());
    assert!(BootPhase::LoadRooms.order() < BootPhase::SpawnEnsigns.order());
    assert!(BootPhase::SpawnEnsigns.order() < BootPhase::ConnectPort.order());
    assert!(BootPhase::ConnectPort.order() < BootPhase::Ready.order());
}

#[test]
fn phase_next_chains_to_ready() {
    let mut p = BootPhase::Init;
    let expected = vec![
        BootPhase::LoadConfig,
        BootPhase::ConnectDB,
        BootPhase::LoadRooms,
        BootPhase::SpawnEnsigns,
        BootPhase::ConnectPort,
        BootPhase::Ready,
    ];
    for exp in expected {
        assert_eq!(p.next(), Some(exp));
        p = exp;
    }
    assert_eq!(p.next(), None);
}

#[test]
fn phase_display() {
    assert_eq!(format!("{}", BootPhase::Init), "Init");
    assert_eq!(format!("{}", BootPhase::LoadConfig), "LoadConfig");
    assert_eq!(format!("{}", BootPhase::ConnectDB), "ConnectDB");
    assert_eq!(format!("{}", BootPhase::LoadRooms), "LoadRooms");
    assert_eq!(format!("{}", BootPhase::SpawnEnsigns), "SpawnEnsigns");
    assert_eq!(format!("{}", BootPhase::ConnectPort), "ConnectPort");
    assert_eq!(format!("{}", BootPhase::Ready), "Ready");
}

#[test]
fn phase_serde_roundtrip() {
    for phase in BootPhase::all() {
        let json = serde_json::to_string(&phase).unwrap();
        let back: BootPhase = serde_json::from_str(&json).unwrap();
        assert_eq!(phase, back);
    }
}

// ── BootError tests ──────────────────────────────────────────────────────────

#[test]
fn boot_error_display() {
    let e = BootError::ConfigMissing("token".into());
    assert_eq!(format!("{e}"), "ConfigMissing: token");
    let e = BootError::DatabaseError("corrupt".into());
    assert_eq!(format!("{e}"), "DatabaseError: corrupt");
    let e = BootError::RoomError("not found".into());
    assert_eq!(format!("{e}"), "RoomError: not found");
    let e = BootError::PortError("timeout".into());
    assert_eq!(format!("{e}"), "PortError: timeout");
}

#[test]
fn boot_error_serde_roundtrip() {
    let errors = vec![
        BootError::ConfigMissing("a".into()),
        BootError::DatabaseError("b".into()),
        BootError::RoomError("c".into()),
        BootError::PortError("d".into()),
    ];
    for e in &errors {
        let json = serde_json::to_string(e).unwrap();
        let back: BootError = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{e}"), format!("{back}"));
    }
}

// ── PortType tests ───────────────────────────────────────────────────────────

#[test]
fn port_type_display() {
    assert_eq!(format!("{}", PortType::Telegram), "Telegram");
    assert_eq!(format!("{}", PortType::Stdio), "Stdio");
    assert_eq!(format!("{}", PortType::Memory), "Memory");
}

#[test]
fn port_type_serde_roundtrip() {
    for pt in [PortType::Telegram, PortType::Stdio, PortType::Memory] {
        let json = serde_json::to_string(&pt).unwrap();
        let back: PortType = serde_json::from_str(&json).unwrap();
        assert_eq!(pt, back);
    }
}

// ── BootConfig tests ─────────────────────────────────────────────────────────

#[test]
fn config_default_values() {
    let c = BootConfig::default();
    assert!(c.telegram_token.is_none());
    assert!(c.deepinfra_key.is_none());
    assert!(c.zai_key.is_none());
    assert_eq!(c.rooms_dir, "rooms/");
    assert_eq!(c.ensigns_dir, "ensigns/");
    assert_eq!(c.db_path, "universe.db");
    assert_eq!(c.rust_log, "info");
}

#[test]
fn config_validate_missing_all() {
    let c = BootConfig::default();
    let w = c.validate();
    assert_eq!(w.len(), 3);
    assert!(w[0].contains("TELEGRAM_TOKEN"));
    assert!(w[1].contains("DEEPINFRA_KEY"));
    assert!(w[2].contains("ZAI_KEY"));
}

#[test]
fn config_validate_all_present() {
    let c = BootConfig {
        telegram_token: Some("tok".into()),
        deepinfra_key: Some("key".into()),
        zai_key: Some("zkey".into()),
        ..BootConfig::default()
    };
    assert!(c.validate().is_empty());
}

#[test]
fn config_validate_partial() {
    let c = BootConfig {
        telegram_token: Some("tok".into()),
        ..BootConfig::default()
    };
    let w = c.validate();
    assert_eq!(w.len(), 2);
}

#[test]
fn config_from_env() {
    std::env::remove_var("TELEGRAM_TOKEN");
    std::env::remove_var("DEEPINFRA_KEY");
    std::env::remove_var("ZAI_KEY");
    std::env::remove_var("ROOMS_DIR");
    std::env::remove_var("ENSIGNS_DIR");
    std::env::remove_var("DB_PATH");
    std::env::remove_var("RUST_LOG");

    let c = BootConfig::from_env();
    assert!(c.telegram_token.is_none());
    assert_eq!(c.rooms_dir, "rooms/");
}

#[test]
fn config_from_env_with_values() {
    std::env::set_var("TELEGRAM_TOKEN", "abc123");
    std::env::set_var("ROOMS_DIR", "my_rooms/");
    let c = BootConfig::from_env();
    assert_eq!(c.telegram_token.as_deref(), Some("abc123"));
    assert_eq!(c.rooms_dir, "my_rooms/");
    std::env::remove_var("TELEGRAM_TOKEN");
    std::env::remove_var("ROOMS_DIR");
}

#[test]
fn config_serde_roundtrip() {
    let c = BootConfig {
        telegram_token: Some("t".into()),
        deepinfra_key: None,
        zai_key: Some("z".into()),
        rooms_dir: "r/".into(),
        ensigns_dir: "e/".into(),
        db_path: "d.db".into(),
        rust_log: "debug".into(),
    };
    let json = serde_json::to_string(&c).unwrap();
    let back: BootConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(c.telegram_token, back.telegram_token);
    assert_eq!(c.zai_key, back.zai_key);
    assert_eq!(c.rooms_dir, back.rooms_dir);
}

// ── BootSequence tests ───────────────────────────────────────────────────────

#[test]
fn sequence_new_starts_at_init() {
    let seq = BootSequence::new(BootConfig::default());
    assert_eq!(seq.phase, BootPhase::Init);
    assert!(!seq.is_ready());
}

#[test]
fn sequence_advance_moves_phase() {
    let mut seq = BootSequence::new(BootConfig::default());
    seq.advance(BootPhase::LoadConfig);
    assert_eq!(seq.phase, BootPhase::LoadConfig);
    seq.advance(BootPhase::ConnectDB);
    assert_eq!(seq.phase, BootPhase::ConnectDB);
}

#[test]
fn sequence_advance_logs() {
    let mut seq = BootSequence::new(BootConfig::default());
    seq.advance(BootPhase::LoadConfig);
    assert!(seq.logs.len() >= 2); // Init log + LoadConfig log
    assert_eq!(seq.logs.last().unwrap().phase, BootPhase::LoadConfig);
}

#[test]
fn sequence_warn_adds_warning() {
    let mut seq = BootSequence::new(BootConfig::default());
    seq.warn("something bad");
    assert_eq!(seq.warnings.len(), 1);
    assert_eq!(seq.warnings[0], "something bad");
}

#[test]
fn sequence_warn_logs() {
    let mut seq = BootSequence::new(BootConfig::default());
    let init_logs = seq.logs.len();
    seq.warn("oops");
    assert_eq!(seq.logs.len(), init_logs + 1);
    assert!(seq.logs.last().unwrap().message.contains("oops"));
}

#[test]
fn sequence_elapsed_is_reasonable() {
    let seq = BootSequence::new(BootConfig::default());
    let elapsed = seq.elapsed_ms();
    assert!(elapsed < 1000); // should be near-instant
}

#[test]
fn sequence_is_ready() {
    let mut seq = BootSequence::new(BootConfig::default());
    assert!(!seq.is_ready());
    seq.advance(BootPhase::Ready);
    assert!(seq.is_ready());
}

#[test]
fn sequence_summary_contains_phase() {
    let seq = BootSequence::new(BootConfig::default());
    let s = seq.summary();
    assert!(s.contains("Init"));
    assert!(s.contains("elapsed"));
    assert!(s.contains("warnings=0"));
}

#[test]
fn sequence_logs_every_phase_transition() {
    let mut seq = BootSequence::new(BootConfig::default());
    for phase in BootPhase::all().iter().skip(1) {
        seq.advance(*phase);
    }
    // Init log + 6 advances
    let phase_names: Vec<String> = seq.logs.iter().map(|l| format!("{}", l.phase)).collect();
    assert!(phase_names.iter().any(|p| p == "Init"));
    assert!(phase_names.iter().any(|p| p == "LoadConfig"));
    assert!(phase_names.iter().any(|p| p == "ConnectDB"));
    assert!(phase_names.iter().any(|p| p == "LoadRooms"));
    assert!(phase_names.iter().any(|p| p == "SpawnEnsigns"));
    assert!(phase_names.iter().any(|p| p == "ConnectPort"));
    assert!(phase_names.iter().any(|p| p == "Ready"));
}

// ── DatabaseBoot tests ───────────────────────────────────────────────────────

#[test]
fn db_bootstrap_creates_tables() {
    let mut db = DatabaseBoot::new("test.db");
    let tables = db.bootstrap().unwrap();
    assert_eq!(tables, vec!["provenance", "tiles", "conservation"]);
    assert_eq!(db.tables_created.len(), 3);
    assert!(db.wal_enabled);
}

#[test]
fn db_bootstrap_empty_path_errors() {
    let mut db = DatabaseBoot::new("");
    let r = db.bootstrap();
    assert!(r.is_err());
}

#[test]
fn db_verify_schema_clean() {
    let mut db = DatabaseBoot::new("test.db");
    db.bootstrap().unwrap();
    let missing = db.verify_schema();
    assert!(missing.is_empty());
}

#[test]
fn db_verify_schema_missing_tables() {
    let db = DatabaseBoot {
        path: "test.db".into(),
        tables_created: vec!["provenance".into()],
        wal_enabled: false,
    };
    let missing = db.verify_schema();
    assert_eq!(missing.len(), 2);
    assert!(missing[0].contains("tiles"));
    assert!(missing[1].contains("conservation"));
}

#[test]
fn db_enable_wal() {
    let mut db = DatabaseBoot::new("test.db");
    db.enable_wal().unwrap();
    assert!(db.wal_enabled);
}

#[test]
fn db_enable_wal_empty_path() {
    let mut db = DatabaseBoot::new("");
    assert!(db.enable_wal().is_err());
}

#[test]
fn db_serde_roundtrip() {
    let db = DatabaseBoot {
        path: "x.db".into(),
        tables_created: vec!["a".into(), "b".into()],
        wal_enabled: true,
    };
    let json = serde_json::to_string(&db).unwrap();
    let back: DatabaseBoot = serde_json::from_str(&json).unwrap();
    assert_eq!(db.path, back.path);
    assert_eq!(db.tables_created, back.tables_created);
    assert_eq!(db.wal_enabled, back.wal_enabled);
}

// ── RoomBoot tests ───────────────────────────────────────────────────────────

#[test]
fn room_boot_new_is_empty() {
    let rb = RoomBoot::new();
    assert!(rb.rooms_loaded.is_empty());
    assert!(rb.ensigns_spawned.is_empty());
}

#[test]
fn room_load_from_dir() {
    let dir = TempDir::new().unwrap();
    let room_path = dir.path().join("main.toml");
    std::fs::write(&room_path, "[room]").unwrap();
    let other = dir.path().join("other.json");
    std::fs::write(&other, "{}").unwrap();

    let mut rb = RoomBoot::new();
    let rooms = rb.load_rooms(dir.path().to_str().unwrap()).unwrap();
    assert_eq!(rooms.len(), 2);
}

#[test]
fn room_load_missing_dir_gives_zero() {
    let mut rb = RoomBoot::new();
    let rooms = rb.load_rooms("/nonexistent/path/").unwrap();
    assert!(rooms.is_empty());
}

#[test]
fn room_load_empty_dir_errors() {
    let mut rb = RoomBoot::new();
    assert!(rb.load_rooms("").is_err());
}

#[test]
fn ensign_spawn_from_dir() {
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("herald.toml");
    std::fs::write(&f, "[ensign]").unwrap();

    let mut rb = RoomBoot::new();
    let ensigns = rb.spawn_ensigns(dir.path().to_str().unwrap()).unwrap();
    assert_eq!(ensigns.len(), 1);
}

#[test]
fn ensign_spawn_missing_dir_gives_zero() {
    let mut rb = RoomBoot::new();
    let ensigns = rb.spawn_ensigns("/nonexistent/").unwrap();
    assert!(ensigns.is_empty());
}

#[test]
fn ensign_spawn_empty_dir_errors() {
    let mut rb = RoomBoot::new();
    assert!(rb.spawn_ensigns("").is_err());
}

#[test]
fn room_summary_singular() {
    let rb = RoomBoot {
        rooms_loaded: vec!["a".into()],
        ensigns_spawned: vec!["b".into()],
    };
    assert_eq!(rb.room_summary(), "1 room, 1 ensign");
}

#[test]
fn room_summary_plural() {
    let rb = RoomBoot {
        rooms_loaded: vec!["a".into(), "b".into(), "c".into()],
        ensigns_spawned: vec!["d".into(), "e".into()],
    };
    assert_eq!(rb.room_summary(), "3 rooms, 2 ensigns");
}

#[test]
fn room_summary_zero() {
    let rb = RoomBoot::new();
    assert_eq!(rb.room_summary(), "0 rooms, 0 ensigns");
}

#[test]
fn room_boot_serde_roundtrip() {
    let rb = RoomBoot {
        rooms_loaded: vec!["r1".into()],
        ensigns_spawned: vec!["e1".into(), "e2".into()],
    };
    let json = serde_json::to_string(&rb).unwrap();
    let back: RoomBoot = serde_json::from_str(&json).unwrap();
    assert_eq!(rb.rooms_loaded, back.rooms_loaded);
    assert_eq!(rb.ensigns_spawned, back.ensigns_spawned);
}

// ── PortBoot tests ───────────────────────────────────────────────────────────

#[test]
fn port_boot_telegram_connect() {
    let mut p = PortBoot::new(PortType::Telegram);
    p.connect().unwrap();
    assert!(p.connected);
    assert!(!p.fallback);
}

#[test]
fn port_boot_stdio_connect() {
    let mut p = PortBoot::new(PortType::Stdio);
    p.connect().unwrap();
    assert!(p.connected);
}

#[test]
fn port_boot_memory_connect() {
    let mut p = PortBoot::new(PortType::Memory);
    p.connect().unwrap();
    assert!(p.connected);
}

#[test]
fn port_boot_fallback_stdio() {
    let mut p = PortBoot::new(PortType::Telegram);
    p.fallback_stdio();
    assert_eq!(p.port_type, PortType::Stdio);
    assert!(p.connected);
    assert!(p.fallback);
}

#[test]
fn port_boot_serde_roundtrip() {
    let p = PortBoot {
        port_type: PortType::Telegram,
        connected: true,
        fallback: false,
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: PortBoot = serde_json::from_str(&json).unwrap();
    assert_eq!(p.port_type, back.port_type);
    assert_eq!(p.connected, back.connected);
    assert_eq!(p.fallback, back.fallback);
}

// ── FullBootSequence tests ───────────────────────────────────────────────────

#[test]
fn full_boot_completes() {
    let config = BootConfig {
        telegram_token: Some("tok".into()),
        deepinfra_key: Some("key".into()),
        zai_key: Some("zkey".into()),
        ..BootConfig::default()
    };
    let result = FullBootSequence::boot(config);
    assert!(result.success);
    assert!(result.sequence.is_ready());
}

#[test]
fn full_boot_completes_under_100ms_simulated() {
    let config = BootConfig {
        telegram_token: Some("tok".into()),
        ..BootConfig::default()
    };
    let result = FullBootSequence::boot(config);
    assert!(result.boot_ms < 1000); // generous for CI
}

#[test]
fn full_boot_missing_telegram_falls_back_stdio() {
    let config = BootConfig {
        telegram_token: None,
        ..BootConfig::default()
    };
    let result = FullBootSequence::boot(config);
    assert!(result.success);
    assert_eq!(result.port_type, PortType::Stdio);
}

#[test]
fn full_boot_with_telegram_uses_telegram() {
    let config = BootConfig {
        telegram_token: Some("tok".into()),
        ..BootConfig::default()
    };
    let result = FullBootSequence::boot(config);
    assert_eq!(result.port_type, PortType::Telegram);
}

#[test]
fn full_boot_missing_rooms_zero_rooms() {
    let config = BootConfig {
        rooms_dir: "/nonexistent/rooms/".into(),
        ..BootConfig::default()
    };
    let result = FullBootSequence::boot(config);
    assert!(result.success);
    assert_eq!(result.rooms_active, 0);
}

#[test]
fn full_boot_missing_ensigns_zero_ensigns() {
    let config = BootConfig {
        ensigns_dir: "/nonexistent/ensigns/".into(),
        ..BootConfig::default()
    };
    let result = FullBootSequence::boot(config);
    assert!(result.success);
    assert_eq!(result.ensigns_active, 0);
}

#[test]
fn full_boot_creates_all_tables() {
    let config = BootConfig::default();
    let result = FullBootSequence::boot(config);
    // Check the DB was bootstrapped (we can see it in the logs)
    let db_logs: Vec<&BootLog> = result
        .sequence
        .logs
        .iter()
        .filter(|l| l.phase == BootPhase::ConnectDB && l.message.contains("tables"))
        .collect();
    assert!(!db_logs.is_empty());
    assert!(db_logs[0].message.contains("3 tables"));
}

#[test]
fn full_boot_logs_every_phase() {
    let config = BootConfig::default();
    let result = FullBootSequence::boot(config);
    let phases: Vec<String> = result
        .sequence
        .logs
        .iter()
        .map(|l| format!("{}", l.phase))
        .collect();
    for phase in BootPhase::all() {
        let name = format!("{phase}");
        assert!(
            phases.iter().any(|p| p == &name),
            "Missing phase log: {name}"
        );
    }
}

#[test]
fn full_boot_empty_config_produces_warnings() {
    let config = BootConfig::default();
    let result = FullBootSequence::boot(config);
    assert!(!result.warnings.is_empty());
    assert!(result.warnings.iter().any(|w| w.contains("TELEGRAM_TOKEN")));
}

#[test]
fn full_boot_result_summary() {
    let config = BootConfig {
        telegram_token: Some("t".into()),
        deepinfra_key: Some("d".into()),
        zai_key: Some("z".into()),
        ..BootConfig::default()
    };
    let result = FullBootSequence::boot(config);
    let s = result.summary();
    assert!(s.contains("success=true"));
    assert!(s.contains("port=Telegram"));
    assert!(s.contains("rooms=0"));
    assert!(s.contains("ensigns=0"));
    assert!(s.contains("warnings=0"));
}

#[test]
fn full_boot_idempotent() {
    let config = BootConfig {
        telegram_token: Some("t".into()),
        deepinfra_key: Some("d".into()),
        zai_key: Some("z".into()),
        ..BootConfig::default()
    };
    let r1 = FullBootSequence::boot(config.clone());
    let r2 = FullBootSequence::boot(config);
    assert_eq!(r1.success, r2.success);
    assert_eq!(r1.rooms_active, r2.rooms_active);
    assert_eq!(r1.ensigns_active, r2.ensigns_active);
    assert_eq!(r1.port_type, r2.port_type);
    assert_eq!(r1.warnings.len(), r2.warnings.len());
}

#[test]
fn full_boot_graceful_degradation() {
    // All keys missing — should still succeed with warnings
    let config = BootConfig::default();
    let result = FullBootSequence::boot(config);
    assert!(result.success);
    assert_eq!(result.warnings.len(), 3);
    assert_eq!(result.port_type, PortType::Stdio);
}

#[test]
fn full_boot_with_room_files() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("room1.toml"), "[room]").unwrap();
    std::fs::write(dir.path().join("room2.json"), "{}").unwrap();

    let config = BootConfig {
        rooms_dir: dir.path().to_str().unwrap().to_string(),
        ..BootConfig::default()
    };
    let result = FullBootSequence::boot(config);
    assert_eq!(result.rooms_active, 2);
}

#[test]
fn full_boot_with_ensign_files() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("ensign1.toml"), "[ensign]").unwrap();

    let config = BootConfig {
        ensigns_dir: dir.path().to_str().unwrap().to_string(),
        ..BootConfig::default()
    };
    let result = FullBootSequence::boot(config);
    assert_eq!(result.ensigns_active, 1);
}

#[test]
fn boot_log_serde_roundtrip() {
    let log = BootLog {
        phase: BootPhase::ConnectDB,
        timestamp: 1234567890,
        message: "test".into(),
    };
    let json = serde_json::to_string(&log).unwrap();
    let back: BootLog = serde_json::from_str(&json).unwrap();
    assert_eq!(log.phase, back.phase);
    assert_eq!(log.timestamp, back.timestamp);
    assert_eq!(log.message, back.message);
}

#[test]
fn boot_result_serde_roundtrip() {
    let config = BootConfig {
        telegram_token: Some("t".into()),
        ..BootConfig::default()
    };
    let result = FullBootSequence::boot(config);
    let json = serde_json::to_string(&result).unwrap();
    let back: BootResult = serde_json::from_str(&json).unwrap();
    assert_eq!(result.success, back.success);
    assert_eq!(result.rooms_active, back.rooms_active);
    assert_eq!(result.port_type, back.port_type);
}

#[test]
fn boot_sequence_serde_roundtrip() {
    let config = BootConfig::default();
    let mut seq = BootSequence::new(config);
    seq.advance(BootPhase::LoadConfig);
    seq.warn("test warning");
    let json = serde_json::to_string(&seq).unwrap();
    let back: BootSequence = serde_json::from_str(&json).unwrap();
    assert_eq!(seq.phase, back.phase);
    assert_eq!(seq.warnings.len(), back.warnings.len());
}
