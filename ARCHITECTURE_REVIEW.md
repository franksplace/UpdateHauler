# Architecture Review

## 1. Trait Default Methods for Save/Restore

**File:** `src/plugins/mod.rs:72-105`

The `Plugin` trait requires every plugin to implement `save()` and `restore()`, but 8 of 15 plugins have identical implementations that log "no save/restore needed" and return `Ok(())`. This produces ~100 lines of boilerplate.

**Affected plugins:** deno, docker, flatpak, gem, os, rustup, snap, vscode

**Pattern in each:**
```rust
async fn save(&self, _config: &Config, _insights: &Insights, logger: &mut Logger) -> Result<()> {
    logger.log("... is managed by ... - no save needed");
    Ok(())
}
```

**Fix:** Add default implementations to the trait:
```rust
async fn save(&self, _config: &Config, _insights: &Insights, logger: &mut Logger) -> Result<()> {
    logger.log("No save needed for this plugin");
    Ok(())
}
async fn restore(&self, _config: &Config, _insights: &Insights, logger: &mut Logger) -> Result<()> {
    logger.log("No restore needed for this plugin");
    Ok(())
}
```
Plugins that actually need save/restore (brew, cargo, npm, nvim, pip, uv) override the defaults.

---

## 2. Unused Crate Dependencies

**File:** `Cargo.toml:24-33`

Four dependencies are pulled in but never used in source:

| Crate | Why unused |
|-------|------------|
| `lazy_static` | No `lazy_static!` macro used anywhere |
| `dirs` | All paths are constructed manually from `$HOME` |
| `regex` | Only `string.contains()` and simple matching used |
| `serde_json` | `--json` output from commands goes to stdout/stderr, never parsed |

Each brings an entire transitive dependency tree, inflating compile times.

**Fix:** Remove from `Cargo.toml`.

---

## 3. Dead Code Hiding Behind `#[allow(dead_code)]`

**File:** Various

Seven `#[allow(dead_code)]` annotations mask unused code:

| Location | Item | Notes |
|----------|------|-------|
| `src/insights.rs:9` | `plat` field | Set but never read |
| `src/insights.rs:15` | `s_arch` field | Set but never read |
| `src/insights.rs:17` | `linux_full_id` field | Set but never read |
| `src/logger.rs:46` | `debug()` method | Never called |
| `src/logger.rs:53` | `info()` method | Never called |
| `src/logger.rs:70` | `cecho()` method | Never called |
| `src/plugins/mod.rs:238` | `run_available_plugins()` | Never called externally |

**Fix:** Remove unused items or add tests that exercise them if they should exist.

---

## 4. `run_cmd` Silently Swallows Execution Errors

**File:** `src/plugins/mod.rs:342-352`

When `duct::cmd(...).run()` returns `Err(e)` (e.g., binary not found, permission denied), `run_cmd` logs the error but **always returns `Ok(())`**:

```rust
Err(e) => {
    if config.show_header {
        if show_error {
            logger.error(&format!("{} → Error: {}", cmd_str, e));
        }
    }
    Ok(())  // <-- Error swallowed
}
```

This means a missing `brew` binary, a failed `npm update`, or a broken `pip` command all silently count as success. The caller has no way to distinguish success from failure.

**Fix:** Return a `Result` that callers can propagate or handle. At minimum, return a boolean success flag.

---

## 5. Config Boilerplate Explosion Per Plugin

**Files:** `src/config.rs`, `src/insights.rs`, `src/main.rs`, `src/plugins/mod.rs`

Adding a single new plugin requires touching **6+ files**:

1. `src/plugins/NAME.rs` — the plugin itself
2. `src/plugins/mod.rs` — `pub mod NAME; pub use NAME::NamePlugin;`
3. `src/config.rs` — add field to `PluginConfig`, add default in `Config::new()`, add match arm in `apply_plugin_enabled()`, add to `generate_sample_yaml()`
4. `src/main.rs` — add import, add to `register_plugins!`, add to `--list-plugins` match block
5. `src/insights.rs` — add `has_NAME` field, add `which()` call
6. Shell completions (bash + zsh) — hardcoded action lists inside `main.rs`
7. `PossibleValuesParser` — hardcoded action list in clap args

This violates the Open/Closed Principle: the system cannot be extended without modifying existing code.

**Fix:** Several strategies could help:
- Replace the `--list-plugins` match with a data-driven lookup from `PluginConfig`
- Drive the `PossibleValuesParser` from the registry (requires runtime clap building)
- Store the "default enabled" map as a `HashMap<&str, bool>` rather than a struct with 14 fields

---

## 6. Duplicate Tokio Runtime for `--list-plugins`

**File:** `src/main.rs:785,800`

```rust
let rt = tokio::runtime::Runtime::new()?;       // line ~785 — created for main flow
if args.list_plugins {
    let rt = tokio::runtime::Runtime::new()?;    // line ~800 — shadows, second allocation
```

A second runtime is unnecessarily created and never dropped properly, wasting ~1MB per invocation of `--list-plugins`.

**Fix:** Reuse the outer `rt` for the `check_available()` calls.

---

## 7. Path Traversal Check Duplicated 8 Times

**File:** `src/main.rs` (multiple locations)

The same 3-line validation is inlined for every `--*-save-file` flag and `--logfile`, `--installdir`, `--completionsdir`:

```rust
if p.components().any(|c| c == std::path::Component::ParentDir) {
    anyhow::bail!("--XXX path contains '..' traversal: {}", ...);
}
```

Meanwhile, `src/config.rs:112` defines `fn has_path_traversal()` which does exactly this but is only used inside `config.rs`, not in `main.rs`.

**Fix:** Use the existing helper consistently. Could also define a `SanitizedPath` newtype.

---

## 8. `cron_enable` Uses Substring Match (False Positive Risk)

**File:** `src/scheduler.rs:61`

```rust
if current_tab.contains(&self.insights.app_abspath.to_string_lossy().to_string()) {
```

This checks if the crontab contains the app path as a **substring**. If the path is `/usr/local/bin/updatehauler`, a crontab line like:

```
# This script lives at /usr/local/bin/updatehauler
```

or an unrelated entry for `/usr/local/bin/updatehauler-v2` would cause the function to incorrectly skip registration. Should check for a line-based match instead.

---

## 9. Missing Plugins from Test Registrations

**File:** `tests/plugins_test.rs`, `tests/plugin_registry_test.rs`

Both test files register a subset of plugins but omit 4:

- `NpmPlugin`
- `PipPlugin`
- `UvPlugin`
- `RunPlugin`

CI never validates that these plugins register correctly or return valid metadata.

**Fix:** Register all plugins in test setup, or use the same `create_plugin_registry()` function from `main.rs`.

---

## 10. Docker Daemon Check Blocks Async Runtime

**File:** `src/plugins/docker.rs:15-25`

```rust
fn daemon_running(logger: &mut Logger) -> bool {
    let result = cmd("docker", &["info"])
        .stdout_capture()
        .stderr_capture()
        .run();  // <-- blocking call inside async context
```

Called from `async fn update()`. `duct::cmd(...).run()` blocks the current thread, which defeats the purpose of an async runtime. If the Docker daemon is unreachable, this could block for the entire timeout duration.

**Fix:** Wrap in `tokio::task::spawn_blocking(...)` or use an async command executor.

---

## 11. Action Lists Hardcoded in 4 Places

**Files:** `src/main.rs`

Every plugin action appears in **4 separate locations** that must be kept in sync:

1. **`PossibleValuesParser`** (line ~597-650) — validates the `[ACTION]` positional arg
2. **`generate_custom_bash_completion()`** (line ~188) — bash `$commands` variable
3. **`generate_custom_zsh_completion()`** (line ~229-267) — zsh `$commands` array
4. **`--list-plugins` match block** (line ~807-822) — determines enabled/disabled status

Adding a single action (e.g., `brew-list`) requires editing **all 4 locations**. Any mismatch between them causes broken completions or incorrect validation.

**Fix:** Drive all 4 from the plugin registry metadata at runtime. The `--list-plugins` match is the highest priority since it's already partially data-driven.

---

## 12. Main.rs Is a God File (1151 Lines)

**File:** `src/main.rs` (entire file)

`main.rs` contains 13 distinct responsibilities:

| Responsibility | Lines | Extract to |
|---------------|-------|------------|
| CLI argument parsing (clap derive) | ~90 | Keep (idiomatic) |
| Config override logic (~25 `if let` blocks) | ~60 | `config.rs` builder |
| Plugin registry creation | ~20 | Already reusable |
| `build_help_text()` | ~70 | `help.rs` |
| `build_plugin_help()` | ~65 | `help.rs` |
| Bash completion generation | ~35 | `completions.rs` |
| Zsh completion generation | ~100 | `completions.rs` |
| Shell completion installer | ~75 | `completions.rs` |
| Notification (`notify_result`) | ~30 | `notify.rs` |
| Log trimming (`trim_logfile`) | ~40 | `maintenance.rs` |
| Default action builder | ~50 | `cli.rs` |
| Action dispatch loop | ~30 | Keep |
| `main()` | ~415 lines total | Remains but delegates |

The `main()` function alone is 413 lines — a textbook god function.

---

## 13. Plugin Metadata Allocated on Every Call

**File:** Every plugin's `get_metadata()` method

Every call constructs a new `PluginMetadata` with cloned `String`s. Since metadata is immutable per plugin type, this is repeated allocation:

```rust
fn get_metadata(&self) -> PluginMetadata {
    PluginMetadata {
        name: "brew".to_string(),
        description: "...".to_string(),
        actions: vec![...],
    }
}
```

Called every time `--list-plugins`, `--help`, `build_plugin_help()`, or action name resolution runs.

**Fix:** Use `OnceLock<PluginMetadata>` or store metadata as a static reference.

---

## 14. Logger Clones the Entire Config

**File:** `src/logger.rs:17`

```rust
config: config.clone(),
```

`Logger::new()` clones the full `Config` struct (which contains `PathBuf`s, `Vec<String>`, etc.). The logger only needs 5 fields:

- `log: PathBuf`
- `datetime: bool`
- `color: bool`
- `use_log: bool`
- `show_header: bool`

Cloning the entire struct is wasteful heap churn, especially since `Logger` is created per test and per plugin call.

**Fix:** Extract a `LoggerConfig` with only the needed fields.

---

## 15. Pip Plugin Uses Shell Pipeline Strings

**File:** `src/plugins/pip.rs:58-72`

```rust
super::run_cmd(config, logger, false, "sh", &[
    "-c",
    "uv pip list --outdated --format=columns 2>/dev/null \
     | tail -n +3 | grep -v '^Using' \
     | awk '{print $1}' \
     | xargs -r uv pip install --upgrade --system --break-system-packages 2>&1 || true",
])?;
```

This constructs a shell pipeline as a raw string. Problems:
- **Shell-dependent:** `tail`, `grep`, `awk`, `xargs` — all must be installed and POSIX-compatible
- **Hard to debug:** If the pipeline fails, no intermediate output is visible
- **Injection risk:** If a package name contains shell metacharacters (e.g., `foo; rm -rf /`), it would be interpreted
- **`|| true`:** Hides all failures including genuine errors

**Fix:** Use `uv pip list --outdated --format=json`, parse the JSON in Rust, and call `uv pip install --upgrade` per package or batch.

---

## 16. `build_help_text()` Uses `Box::leak`

**File:** `src/main.rs:91`

```rust
Box::leak(help.into_boxed_str())
```

Clap's `after_help` requires a `&'static str`, so leaking is the pragmatic solution. It's called once per program lifetime, so it's not a resource leak in practice — but it sets a precedent that's easy to copy incorrectly.

**Fix:** A `OnceLock<String>` would be cleaner, or avoid the `after_help` attribute entirely by printing help manually.

---

## 17. `trim_logfile` Reads Entire File Twice Into Memory

**File:** `src/main.rs:1121-1134`

```rust
let line_count = reader.lines().count();      // First full read
...
let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();  // Second full read
```

The log file is read in its entirety twice, and all lines are stored in a `Vec<String>`. For a 100MB log file, this uses 200MB+ of peak memory.

**Fix:** Use a ring-buffer approach: read lines into a `VecDeque<String>` of capacity `max_lines`, discarding old lines.

---

## 18. Multiple Registries Created per Invocation

**File:** `src/main.rs`

```rust
build_help_text() { create_plugin_registry(); ... }  // Registry #1
build_plugin_help() { create_plugin_registry(); ... }  // Registry #2
main() { let plugin_registry = create_plugin_registry(); ... }  // Registry #3
```

Each call to `create_plugin_registry()` constructs 15 plugin instances and registers them. These are identical and could be cached.

**Fix:** Store a static registry via `OnceLock` or pass it as a parameter.

---

## 19. Insights Tests Have No Assertions

**File:** `tests/insights_test.rs`

```rust
fn test_brew_detection() {
    let insights = Insights::new().expect("...");
    let _has_brew = insights.has_brew;  // no assertion!
}
fn test_cargo_detection() {
    let insights = Insights::new().expect("...");
    let _has_cargo = insights.has_cargo;  // no assertion!
}
```

These tests bind the value to `_` and never assert anything. They pass trivially regardless of whether detection works.

**Fix:** Add meaningful assertions or remove the tests.

---

## 20. Schedule Args Test Uses Wrong Build Profile

**File:** `tests/schedule_args_test.rs:8`

```rust
path.push("target/release/updatehauler");
```

This points to `target/release/` while all other integration tests use `target/debug/`. During `cargo test` (debug builds), this path won't exist and tests silently skip.

**Fix:** Use `target/debug/` like every other test, or derive the path from `env!("CARGO_MANIFEST_DIR")`.

---

## Summary

### Priority Matrix

| Priority | Effort | Changes |
|----------|--------|---------|
| **P0 — Fix bugs** | 30min | #4 (run_cmd errors), #8 (cron substring), #20 (wrong build profile) |
| **P1 — Reduce boilerplate** | 30min | #1 (trait defaults), #3 (dead code), #6 (duplicate runtime) |
| **P2 — Improve test coverage** | 30min | #9 (missing plugins), #19 (no-assert tests) |
| **P3 — Remove waste** | 30min | #2 (unused deps), #7 (path traversal helper) |
| **P4 — Major refactors** | 4-12h | #5 (config explosion), #11 (dynamic completions), #12 (extract main.rs) |
| **P5 — Performance** | 2+ hours | #10 (blocking in async), #13 (metadata allocation), #14 (logger config), #17 (log trimming) |
