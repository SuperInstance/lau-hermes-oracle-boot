# lau-hermes-oracle-boot

A **boot sequence simulator** for hermes-construct on Oracle ARM — models every phase from cold-start to "Ready", with structured logging, graceful degradation, and full serde serialisation.

## What This Does

When a hermes-construct instance starts up on Oracle's ARM infrastructure, it must:

1. **Read configuration** from environment variables and files
2. **Bootstrap the database** (create tables, enable WAL)
3. **Load rooms** (PLATO rooms from `.toml`/`.json` files)
4. **Spawn ensigns** (sub-agents from config files)
5. **Connect a port** (Telegram, stdio, or in-memory)
6. **Report ready**

This library simulates that exact sequence. Every phase transition is logged with timestamps, every failure degrades gracefully with a warning, and the entire boot state is serialisable for diagnostics or replay.

## Key Idea

Boot is a **state machine** with seven phases, each advancing to the next. The `FullBootSequence::boot()` function drives the machine from `Init` to `Ready`, collecting `BootLog` entries and `BootError` warnings along the way. Nothing panics — errors become warnings, and the system always reaches a terminal state.

```
Init → LoadConfig → ConnectDB → LoadRooms → SpawnEnsigns → ConnectPort → Ready
```

## Install

```toml
[dependencies]
lau-hermes-oracle-boot = "0.1"
```

Or:

```bash
cargo add lau-hermes-oracle-boot
```

Requires **Rust 2021 edition**.

## Quick Start

```rust
use lau_hermes_oracle_boot::{BootConfig, FullBootSequence};

fn main() {
    // Configure via environment variables or manually
    let config = BootConfig::from_env();
    // Or build manually:
    // let config = BootConfig { telegram_token: Some("...".into()), ..Default::default() };

    // Run the full boot sequence
    let result = FullBootSequence::boot(config);

    println!("{}", result.summary());
    // → BootResult(success=true, 12ms, rooms=3, ensigns=2, port=Stdio, warnings=2)

    for log in &result.sequence.logs {
        println!("[{}] {:?}: {}", log.timestamp, log.phase, log.message);
    }
}
```

## API Reference

### `BootPhase`

The seven phases of boot, in order:

| Phase | Order | Description |
|-------|-------|-------------|
| `Init` | 0 | Entry point |
| `LoadConfig` | 1 | Read environment and config files |
| `ConnectDB` | 2 | Bootstrap database tables and WAL |
| `LoadRooms` | 3 | Scan and load PLATO room configs |
| `SpawnEnsigns` | 4 | Spin up sub-agent processes |
| `ConnectPort` | 5 | Establish I/O port |
| `Ready` | 6 | Boot complete |

Methods: `all()`, `order()`, `next()`, `Display`.

### `BootError`

```rust
pub enum BootError {
    ConfigMissing(String),
    DatabaseError(String),
    RoomError(String),
    PortError(String),
}
```

Implements `std::error::Error` and `Display`.

### `BootConfig`

Configuration for the boot sequence:

| Field | Type | Default |
|-------|------|---------|
| `telegram_token` | `Option<String>` | `None` |
| `deepinfra_key` | `Option<String>` | `None` |
| `zai_key` | `Option<String>` | `None` |
| `rooms_dir` | `String` | `"rooms/"` |
| `ensigns_dir` | `String` | `"ensigns/"` |
| `db_path` | `String` | `"universe.db"` |
| `rust_log` | `String` | `"info"` |

- `from_env()` — reads from `TELEGRAM_TOKEN`, `DEEPINFRA_KEY`, `ZAI_KEY`, etc.
- `validate()` — returns warnings for missing optional keys.

### `BootSequence`

The stateful boot tracker. Holds the current phase, start time, warnings, and log entries.

| Method | Description |
|--------|-------------|
| `new(config)` | Create at `Init` phase |
| `advance(phase)` | Transition to a new phase (logs the transition) |
| `warn(msg)` | Append a warning (also logged) |
| `elapsed_ms()` | Milliseconds since boot start |
| `is_ready()` | `true` if phase is `Ready` |
| `summary()` | Human-readable one-liner |

### `DatabaseBoot`

Simulates database bootstrap:

| Method | Description |
|--------|-------------|
| `new(path)` | Create with a db path |
| `bootstrap()` | Creates tables (`provenance`, `tiles`, `conservation`), enables WAL |
| `verify_schema()` | Returns missing table names |
| `enable_wal()` | Explicitly enable write-ahead logging |

### `RoomBoot`

Loads rooms and spawns ensigns from filesystem directories:

| Method | Description |
|--------|-------------|
| `load_rooms(dir)` | Scan dir for `.toml`/`.json` files |
| `spawn_ensigns(dir)` | Scan dir for ensign configs |
| `room_summary()` | `"N rooms, M ensigns"` |

Missing directories are **not errors** — they just result in zero loaded items.

### `PortBoot`

Manages I/O port connection:

| Method | Description |
|--------|-------------|
| `new(port_type)` | Create for `Telegram`, `Stdio`, or `Memory` |
| `connect()` | Simulate connection (always succeeds for simulated types) |
| `fallback_stdio()` | Switch to stdio after a failed connection |

### `BootResult`

Final output of `FullBootSequence::boot()`:

```rust
pub struct BootResult {
    pub success: bool,
    pub sequence: BootSequence,
    pub boot_ms: u64,
    pub rooms_active: usize,
    pub ensigns_active: usize,
    pub port_type: PortType,
    pub warnings: Vec<String>,
}
```

- `summary()` — `"BootResult(success=true, 12ms, rooms=3, ensigns=2, port=Stdio, warnings=0)"`

### `FullBootSequence`

Static entry point with a single method:

```rust
FullBootSequence::boot(config) → BootResult
```

Drives all six phase transitions in order.

## How It Works

### Phase Machine

`FullBootSequence::boot()` executes these steps:

1. **Init** — Create `BootSequence`, record start time.
2. **LoadConfig** — Validate config; missing API keys become warnings (not errors).
3. **ConnectDB** — `DatabaseBoot::bootstrap()` creates three tables (`provenance`, `tiles`, `conservation`) and enables WAL.
4. **LoadRooms** — `RoomBoot::load_rooms()` scans the configured directory.
5. **SpawnEnsigns** — `RoomBoot::spawn_ensigns()` scans the ensigns directory.
6. **ConnectPort** — If `telegram_token` is set, use `Telegram`; otherwise `Stdio`. Failure triggers `fallback_stdio()`.
7. **Ready** — Record elapsed time, return `BootResult`.

### Graceful Degradation

No phase failure aborts the boot. Instead:
- Database errors → warning + continue
- Missing room directory → 0 rooms + continue
- Port connection failure → fallback to stdio

This matches the real hermes-construct philosophy: always reach `Ready`, even if degraded.

### Port Selection Logic

```
if telegram_token is set → Telegram
else                      → Stdio
```

If Telegram connection fails, the system falls back to stdio with `fallback = true`.

## The Math

This library is operational rather than mathematical, but it does enforce a **total ordering** on boot phases:

$$
\text{Init} < \text{LoadConfig} < \text{ConnectDB} < \text{LoadRooms} < \text{SpawnEnsigns} < \text{ConnectPort} < \text{Ready}
$$

The boot duration is:

$$
t_{\text{boot}} = t_{\text{Ready}} - t_{\text{Init}}
$$

Both measured in milliseconds since Unix epoch.

## Testing

65 tests covering phase transitions, config validation, database bootstrap, room loading, port selection, serde round-trips, and the full boot sequence end-to-end.

```bash
cargo test
```

## License

MIT
