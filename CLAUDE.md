# CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes. Bias toward caution over speed; for trivial tasks, use judgment. Merge with project-specific instructions as needed.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

- State assumptions explicitly; if uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop, name what's confusing, and ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked; no abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it. Would a senior engineer call it overcomplicated? Then simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

- Don't "improve" adjacent code, comments, or formatting; don't refactor what isn't broken; match existing style, even if you'd do it differently.
- Remove imports/variables/functions YOUR changes orphaned — but don't delete pre-existing dead code; mention it instead.
- The test: every changed line traces directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

- Turn tasks into verifiable goals: "add validation" → write tests for invalid inputs, then pass them; "fix the bug" → write a reproducing test first; "refactor X" → tests pass before and after.
- For multi-step tasks, state a brief plan as `step → verify: check` lines.
- Strong criteria let you loop independently; weak criteria ("make it work") require constant clarification.

## 5. Testing Discipline

**E2E tests are the highest-priority signal. Cover the real user journey. Never silence failures.**

- Prioritize E2E/integration over unit when coverage is limited; design cases around the user's real path and don't skip steps.
- For CLI tools, test via the binary with real commands — no stubbed subprocesses or mocked file systems.
- Don't use mock data in E2E; run against real system tools (brew, cargo, npm, etc.) or dry-run mode. If mocking seems unavoidable, stop and get human confirmation first.
- Never skip, disable, or `.only` a test to go green — investigate the underlying bug instead.
- E2E tests must be **source-blind**: design assertions from scenario reasonableness alone, never by reading product source to pick expected values. The test verifies the observable contract, not the implementation.
- **If an E2E test fails, the default conclusion is a bug in the code, not the test.** Fix the product; don't weaken the assertion, relax the expected value, change the scenario, or read the source to explain it away. Only change a failing test if a human confirms the scenario is invalid.
- If a test case itself looks wrong, flag it and ask a human — don't silently delete or rewrite it.

## 6. Research Discipline

**Verify against primary sources. Never guess or infer product behavior.**

- Confirm details only via **official documentation** and **source code**; don't speculate or fill gaps with assumptions.
- If docs and source don't answer it, say so and ask — don't invent an answer.
- Cite the specific doc URL, file path, or commit/version for any claim about third-party behavior.

## 7. Reference Implementations Before Building

**Before implementing any feature, study how established players did it — don't drift from the ecosystem.**

Before writing the first line of a new feature, read:

- **Mainstream CLI tool implementations** — research at least three established, similar CLI tools and study how each solves the problem. When in doubt about a command structure or flag, read their sources and compare.
- **Upstream tool docs** — the authoritative spec for any external tool: Homebrew <https://docs.brew.sh/>, Cargo <https://doc.rust-lang.org/cargo/>, npm <https://docs.npmjs.com/>, pip <https://pip.pypa.io/>, uv <https://docs.astral.sh/uv/>.
- **Rust CLI ecosystem** — clap <https://docs.rs/clap/>, duct <https://docs.rs/duct/>, colored <https://docs.rs/colored/>, serde <https://serde.rs/>.

The rule:

- For any new plugin, command, or flag, compare how at least three mainstream CLI tools approach it, cite one upstream-spec source, and summarize that comparison — plus where your design lands — in the design notes / PR description.
- If your design diverges from how those tools solve it, name the divergence and justify it ("they do X but we need Y because of Z" — not "I didn't know they handled it").
- For any flag, env var, or exit code you emit or parse, cite the upstream doc URL or SDK file/line. Don't invent names the ecosystem has already chosen.

## 8. Independent Audit Before Merge

**Every PR pushed must be reviewed by an independent audit agent. Merge is blocked until all HIGH/MEDIUM findings are resolved or explicitly justified.**

After every `gh pr create` or force-push, spawn a fresh `general-purpose` Agent with no shared context. Brief it cold with the PR URL and the contract the PR claims to pin. Treat each angle as blocking:

- **Correctness** — does it do what the description claims? Would a real regression fail the assertions?
- **Reliability** — races, error handling, retry/timeout, propagation timing on slow CI.
- **Security** — input validation at boundaries, command injection, shell escaping, sudo usage, credential handling.
- **Sensitive-info leakage** — secrets in logs/errors, tokens/PII in config files or backups.
- **Breaking changes** — CLI interface, config file format, default behavior shifts; if breaking, is it gated/versioned?
- **E2E coverage** — the user-visible contract, not just unit happy-path; mocks tight enough that a regression on the unverified side can't sneak through.

Output HIGH/MEDIUM/LOW per finding with **concrete suggested code**, not vague "consider". **Merge gate:** every HIGH and MEDIUM is either fixed in code or explicitly justified in the PR (e.g. "feature gap, filed as #N, agreed not to block"); silent merge is not enough. For findings that surface product-behavior gaps, file separate issues and link them. Self-review misses the author's blind spots — an independent agent catches them.

## Plugin Families Stay in Lockstep

**The plugin handlers come in families that share dispatch, config, logging, and error handling logic — brew, cargo, npm, pip, uv, nvim, os, deno, docker, flatpak, gem, go, rustup, snap, vscode, yarn. A bug or feature landed on one almost always applies to the others, and a gap on the unfixed siblings is SILENT: nothing errors, the behavior just quietly degrades.**

- When you touch a per-plugin mechanism (a config option, a save/restore pattern, an action handler, a dry-run check), grep the offending call/pattern across the whole crate and wire **every** sibling plugin in the same PR — or state explicitly in the PR which sibling is deferred and why, and file the follow-up issue immediately.
- "Documented follow-up" without an issue is how gaps rot: it lives in one PR description and no one ever comes back.
- Test coverage must include each wired plugin, not just one: an e2e that only drives `brew` will stay green forever while `cargo` or `npm` traffic silently misbehaves.
- Prefer hoisting the shared logic into one chokepoint (e.g. `Plugin::run_action`) so the family can't drift again.

## A Config Knob Isn't Shipped Until the CLI Exposes It

**A user-configurable feature is NOT delivered when the Rust side works — it's delivered when a user can reach it through the CLI or config file.**

- **Treat any PR that adds or extends a user-facing config surface as automatically implying CLI flag + YAML config + documentation.** The implementation is not "done" on its own; it's one half of a user-facing feature.
- **"Done" for such a feature spans three layers**, none optional: (1) the CLI flag definition in clap; (2) the YAML config schema in `config.yaml`; (3) the README/docs update with examples.
- **If you can only do the implementation half in this PR, say so and file/track the CLI/config/docs issue in the same breath** — never let the umbrella task close on implementation-only work. A merged PR with no user-facing exposure is a latent gap, not a shipped feature.
- Pure internal mechanics (a new algorithm with no user-set config, an observability metric, an internal refactor) don't need CLI work — this rule is specifically about **user-configurable** surfaces a customer must be able to set.

## The Config Model Is Canonical in config.yaml

**When this repo's code and the default config.yaml disagree about a field's name, type, or nesting, the config.yaml wins by definition — the code converges to it.**

- Adding or renaming a user-facing config field starts by defining its name and shape in `config.yaml` (and the corresponding `Config` struct); the Rust model then implements exactly that name. The naming decision happens once, in the config — never independently in code.
- Renames use `#[serde(alias = "…")]` so existing config files keep loading through the deprecation window; never hard-rename a shipped field in one step.
- Why the config.yaml and not the code: the config file is the user-facing contract, it renders into the `--config-file` example, and the code's structs are generated from the implementation — a struct that follows the implementation cannot lead it.

## Documentation Lives in README.md

**User-facing documentation is maintained in the README.md, not in a separate docs/ directory.**

- This repo's source tree intentionally carries **no** separate user-facing doc pages — the README is the single source of truth. Do not add or keep prose docs under `docs/` here.
- When a feature needs documentation, update the README.md — never introduce a `docs/*.md` page in this repo, even temporarily or "just for now".
- Code-level doc comments stay with the code — including `///` comments on public APIs.

---

## Project-Specific Notes for updatehauler

### Architecture
- **Plugin-based CLI** — each package manager is a plugin (`brew`, `cargo`, `npm`, `pip`, `uv`, `nvim`, `os`, `deno`, `docker`, `flatpak`, `gem`, `go`, `rustup`, `snap`, `vscode`, `yarn`)
- **Async runtime** — Tokio-based, plugins implement `Plugin` trait
- **Config** — YAML at `~/.config/updatehauler/config.yaml`, CLI flags override
- **Dry-run mode** — `--dry-run` flag for CI/CD testing, must not prompt for passwords or modify system

### Key Files
- `src/main.rs` — CLI entry, plugin dispatch
- `src/lib.rs` — Core types, Plugin trait, config
- `src/plugins/*.rs` — Individual plugin implementations
- `src/config.rs` — Config loading/merging
- `test_release.sh` — Release test suite (must pass before release)
- `.github/workflows/*.yml` — CI/CD (pr.yml, pr-validation.yml, coverage.yml, release.yml)

### Testing
- Unit tests: `cargo test`
- Integration/E2E: `./test_release.sh` (tests binary, flags, dry-run, error handling)
- Coverage: `cargo tarpaulin` (via coverage.yml workflow)
- Dry-run tests in CI verify no password prompts, no system modifications

### Release Process
1. Bump version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Update `test_release.sh` version check
4. Commit, tag, push to main
5. CI builds, tests, runs release test suite, publishes to crates.io

### Common Patterns
- **Plugins**: implement `Plugin` trait with `name()`, `actions()`, `run_action()`
- **Actions**: each plugin defines actions (update, save, restore, list, outdated, etc.)
- **Config**: global + per-plugin, merged with CLI override precedence
- **Errors**: use `anyhow::Result`, return user-facing errors via `anyhow::bail!`
- **Logging**: `tracing` with `--debug` flag, logfile rotation via `--max-log-lines`

### CI Triggers
- PRs to `main`/`develop` → pr.yml + pr-validation.yml + coverage.yml
- Push to `main` → release.yml
- All workflows ignore `**.md` and `**.markdown` changes

---

**Working if:** fewer unnecessary diff lines, fewer overcomplication rewrites, and clarifying questions come before implementation rather than after mistakes.