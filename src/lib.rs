use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

// ── BootPhase ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootPhase {
    Init,
    LoadConfig,
    ConnectDB,
    LoadRooms,
    SpawnEnsigns,
    ConnectPort,
    Ready,
}

impl BootPhase {
    pub fn all() -> Vec<BootPhase> {
        vec![
            BootPhase::Init,
            BootPhase::LoadConfig,
            BootPhase::ConnectDB,
            BootPhase::LoadRooms,
            BootPhase::SpawnEnsigns,
            BootPhase::ConnectPort,
            BootPhase::Ready,
        ]
    }

    pub fn order(&self) -> u8 {
        match self {
            BootPhase::Init => 0,
            BootPhase::LoadConfig => 1,
            BootPhase::ConnectDB => 2,
            BootPhase::LoadRooms => 3,
            BootPhase::SpawnEnsigns => 4,
            BootPhase::ConnectPort => 5,
            BootPhase::Ready => 6,
        }
    }

    pub fn next(&self) -> Option<BootPhase> {
        match self {
            BootPhase::Init => Some(BootPhase::LoadConfig),
            BootPhase::LoadConfig => Some(BootPhase::ConnectDB),
            BootPhase::ConnectDB => Some(BootPhase::LoadRooms),
            BootPhase::LoadRooms => Some(BootPhase::SpawnEnsigns),
            BootPhase::SpawnEnsigns => Some(BootPhase::ConnectPort),
            BootPhase::ConnectPort => Some(BootPhase::Ready),
            BootPhase::Ready => None,
        }
    }
}

impl fmt::Display for BootPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BootPhase::Init => write!(f, "Init"),
            BootPhase::LoadConfig => write!(f, "LoadConfig"),
            BootPhase::ConnectDB => write!(f, "ConnectDB"),
            BootPhase::LoadRooms => write!(f, "LoadRooms"),
            BootPhase::SpawnEnsigns => write!(f, "SpawnEnsigns"),
            BootPhase::ConnectPort => write!(f, "ConnectPort"),
            BootPhase::Ready => write!(f, "Ready"),
        }
    }
}

// ── BootError ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BootError {
    ConfigMissing(String),
    DatabaseError(String),
    RoomError(String),
    PortError(String),
}

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BootError::ConfigMissing(s) => write!(f, "ConfigMissing: {s}"),
            BootError::DatabaseError(s) => write!(f, "DatabaseError: {s}"),
            BootError::RoomError(s) => write!(f, "RoomError: {s}"),
            BootError::PortError(s) => write!(f, "PortError: {s}"),
        }
    }
}

impl std::error::Error for BootError {}

// ── PortType ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortType {
    Telegram,
    Stdio,
    Memory,
}

impl fmt::Display for PortType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortType::Telegram => write!(f, "Telegram"),
            PortType::Stdio => write!(f, "Stdio"),
            PortType::Memory => write!(f, "Memory"),
        }
    }
}

// ── BootLog ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootLog {
    pub phase: BootPhase,
    pub timestamp: u64,
    pub message: String,
}

// ── BootConfig ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootConfig {
    pub telegram_token: Option<String>,
    pub deepinfra_key: Option<String>,
    pub zai_key: Option<String>,
    pub rooms_dir: String,
    pub ensigns_dir: String,
    pub db_path: String,
    pub rust_log: String,
}

impl Default for BootConfig {
    fn default() -> Self {
        Self {
            telegram_token: None,
            deepinfra_key: None,
            zai_key: None,
            rooms_dir: "rooms/".to_string(),
            ensigns_dir: "ensigns/".to_string(),
            db_path: "universe.db".to_string(),
            rust_log: "info".to_string(),
        }
    }
}

impl BootConfig {
    pub fn from_env() -> Self {
        Self {
            telegram_token: std::env::var("TELEGRAM_TOKEN").ok(),
            deepinfra_key: std::env::var("DEEPINFRA_KEY").ok(),
            zai_key: std::env::var("ZAI_KEY").ok(),
            rooms_dir: std::env::var("ROOMS_DIR").unwrap_or_else(|_| "rooms/".to_string()),
            ensigns_dir: std::env::var("ENSIGNS_DIR").unwrap_or_else(|_| "ensigns/".to_string()),
            db_path: std::env::var("DB_PATH").unwrap_or_else(|_| "universe.db".to_string()),
            rust_log: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if self.telegram_token.is_none() {
            warnings.push("TELEGRAM_TOKEN not set — will fall back to stdio".to_string());
        }
        if self.deepinfra_key.is_none() {
            warnings.push("DEEPINFRA_KEY not set — AI features unavailable".to_string());
        }
        if self.zai_key.is_none() {
            warnings.push("ZAI_KEY not set — some features unavailable".to_string());
        }
        warnings
    }
}

// ── BootSequence ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootSequence {
    pub phase: BootPhase,
    pub started_at: u64,
    pub warnings: Vec<String>,
    pub logs: Vec<BootLog>,
    pub config: BootConfig,
}

impl BootSequence {
    pub fn new(config: BootConfig) -> Self {
        let now = epoch_ms();
        let mut seq = Self {
            phase: BootPhase::Init,
            started_at: now,
            warnings: Vec::new(),
            logs: Vec::new(),
            config,
        };
        seq.log(BootPhase::Init, "Boot sequence initialized".to_string());
        seq
    }

    pub fn advance(&mut self, phase: BootPhase) {
        self.log(phase, format!("Phase transition → {phase}"));
        self.phase = phase;
    }

    pub fn warn(&mut self, msg: &str) {
        self.warnings.push(msg.to_string());
        self.log(self.phase, format!("WARNING: {msg}"));
    }

    pub fn elapsed_ms(&self) -> u64 {
        epoch_ms().saturating_sub(self.started_at)
    }

    pub fn is_ready(&self) -> bool {
        self.phase == BootPhase::Ready
    }

    pub fn summary(&self) -> String {
        let phase_list: Vec<String> = self.logs.iter().map(|l| format!("{}", l.phase)).collect();
        format!(
            "BootSequence(phase={}, elapsed={}ms, phases=[{}], warnings={})",
            self.phase,
            self.elapsed_ms(),
            phase_list.join(", "),
            self.warnings.len()
        )
    }

    fn log(&mut self, phase: BootPhase, message: String) {
        self.logs.push(BootLog {
            phase,
            timestamp: epoch_ms(),
            message,
        });
    }
}

// ── DatabaseBoot ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseBoot {
    pub path: String,
    pub tables_created: Vec<String>,
    pub wal_enabled: bool,
}

impl DatabaseBoot {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            tables_created: Vec::new(),
            wal_enabled: false,
        }
    }

    pub fn bootstrap(&mut self) -> Result<Vec<String>, BootError> {
        if self.path.is_empty() {
            return Err(BootError::DatabaseError("db_path is empty".to_string()));
        }

        let tables = vec![
            "provenance".to_string(),
            "tiles".to_string(),
            "conservation".to_string(),
        ];

        for table in &tables {
            self.tables_created.push(table.clone());
        }

        self.wal_enabled = true;
        Ok(tables)
    }

    pub fn verify_schema(&self) -> Vec<String> {
        let required = ["provenance", "tiles", "conservation"];
        required
            .iter()
            .filter(|t| !self.tables_created.contains(&t.to_string()))
            .map(|t| format!("Missing table: {t}"))
            .collect()
    }

    pub fn enable_wal(&mut self) -> Result<(), BootError> {
        if self.path.is_empty() {
            return Err(BootError::DatabaseError(
                "Cannot enable WAL: no database path".to_string(),
            ));
        }
        self.wal_enabled = true;
        Ok(())
    }
}

// ── RoomBoot ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomBoot {
    pub rooms_loaded: Vec<String>,
    pub ensigns_spawned: Vec<String>,
}

impl Default for RoomBoot {
    fn default() -> Self {
        Self::new()
    }
}

impl RoomBoot {
    pub fn new() -> Self {
        Self {
            rooms_loaded: Vec::new(),
            ensigns_spawned: Vec::new(),
        }
    }

    pub fn load_rooms(&mut self, dir: &str) -> Result<Vec<String>, BootError> {
        if dir.is_empty() {
            return Err(BootError::RoomError("rooms_dir is empty".to_string()));
        }

        // Simulate scanning the directory
        if std::path::Path::new(dir).is_dir() {
            let entries = std::fs::read_dir(dir)
                .map_err(|e| BootError::RoomError(format!("Cannot read rooms dir: {e}")))?;
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".toml") || name.ends_with(".json") {
                        self.rooms_loaded.push(name.to_string());
                    }
                }
            }
        }
        // Missing dir = 0 rooms, not an error
        Ok(self.rooms_loaded.clone())
    }

    pub fn spawn_ensigns(&mut self, dir: &str) -> Result<Vec<String>, BootError> {
        if dir.is_empty() {
            return Err(BootError::RoomError("ensigns_dir is empty".to_string()));
        }

        if std::path::Path::new(dir).is_dir() {
            let entries = std::fs::read_dir(dir)
                .map_err(|e| BootError::RoomError(format!("Cannot read ensigns dir: {e}")))?;
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".toml") || name.ends_with(".json") {
                        self.ensigns_spawned.push(name.to_string());
                    }
                }
            }
        }
        // Missing dir = 0 ensigns, not an error
        Ok(self.ensigns_spawned.clone())
    }

    pub fn room_summary(&self) -> String {
        format!(
            "{} room{}, {} ensign{}",
            self.rooms_loaded.len(),
            if self.rooms_loaded.len() == 1 { "" } else { "s" },
            self.ensigns_spawned.len(),
            if self.ensigns_spawned.len() == 1 { "" } else { "s" },
        )
    }
}

// ── PortBoot ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortBoot {
    pub port_type: PortType,
    pub connected: bool,
    pub fallback: bool,
}

impl PortBoot {
    pub fn new(port_type: PortType) -> Self {
        Self {
            port_type,
            connected: false,
            fallback: false,
        }
    }

    pub fn connect(&mut self) -> Result<(), BootError> {
        match self.port_type {
            PortType::Telegram => {
                // Simulate: in a real system this would connect to Telegram API
                self.connected = true;
                Ok(())
            }
            PortType::Stdio => {
                self.connected = true;
                Ok(())
            }
            PortType::Memory => {
                self.connected = true;
                Ok(())
            }
        }
    }

    pub fn fallback_stdio(&mut self) {
        self.port_type = PortType::Stdio;
        self.connected = true;
        self.fallback = true;
    }
}

// ── BootResult ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootResult {
    pub success: bool,
    pub sequence: BootSequence,
    pub boot_ms: u64,
    pub rooms_active: usize,
    pub ensigns_active: usize,
    pub port_type: PortType,
    pub warnings: Vec<String>,
}

impl BootResult {
    pub fn summary(&self) -> String {
        format!(
            "BootResult(success={}, {}ms, rooms={}, ensigns={}, port={}, warnings={})",
            self.success,
            self.boot_ms,
            self.rooms_active,
            self.ensigns_active,
            self.port_type,
            self.warnings.len(),
        )
    }
}

// ── FullBootSequence ─────────────────────────────────────────────────────────

pub struct FullBootSequence;

impl FullBootSequence {
    pub fn boot(config: BootConfig) -> BootResult {
        let start = epoch_ms();
        let mut sequence = BootSequence::new(config.clone());

        // 1. Init → LoadConfig
        sequence.advance(BootPhase::LoadConfig);
        let config_warnings = config.validate();
        for w in &config_warnings {
            sequence.warn(w);
        }

        // 2. LoadConfig → ConnectDB
        sequence.advance(BootPhase::ConnectDB);
        let mut db = DatabaseBoot::new(&config.db_path);
        match db.bootstrap() {
            Ok(tables) => {
                sequence.log(
                    BootPhase::ConnectDB,
                    format!("Created {} tables: {}", tables.len(), tables.join(", ")),
                );
            }
            Err(e) => {
                sequence.warn(&format!("Database bootstrap failed: {e}"));
            }
        }

        // 3. ConnectDB → LoadRooms
        sequence.advance(BootPhase::LoadRooms);
        let mut room_boot = RoomBoot::new();
        match room_boot.load_rooms(&config.rooms_dir) {
            Ok(rooms) => {
                sequence.log(
                    BootPhase::LoadRooms,
                    format!("Loaded {} rooms", rooms.len()),
                );
            }
            Err(e) => {
                sequence.warn(&format!("Room loading failed: {e}"));
            }
        }

        // 4. LoadRooms → SpawnEnsigns
        sequence.advance(BootPhase::SpawnEnsigns);
        match room_boot.spawn_ensigns(&config.ensigns_dir) {
            Ok(ensigns) => {
                sequence.log(
                    BootPhase::SpawnEnsigns,
                    format!("Spawned {} ensigns", ensigns.len()),
                );
            }
            Err(e) => {
                sequence.warn(&format!("Ensign spawning failed: {e}"));
            }
        }

        // 5. SpawnEnsigns → ConnectPort
        sequence.advance(BootPhase::ConnectPort);
        let port_type = if config.telegram_token.is_some() {
            PortType::Telegram
        } else {
            PortType::Stdio
        };
        let mut port = PortBoot::new(port_type);
        if port.connect().is_err() {
            sequence.warn("Port connection failed, falling back to stdio");
            port.fallback_stdio();
        }
        sequence.log(
            BootPhase::ConnectPort,
            format!("Connected via {}", port.port_type),
        );

        // 6. ConnectPort → Ready
        sequence.advance(BootPhase::Ready);
        let boot_ms = epoch_ms().saturating_sub(start);

        BootResult {
            success: sequence.is_ready(),
            sequence,
            boot_ms,
            rooms_active: room_boot.rooms_loaded.len(),
            ensigns_active: room_boot.ensigns_spawned.len(),
            port_type: port.port_type,
            warnings: config_warnings,
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
