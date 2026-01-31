# Repository Guidelines

## Project Structure & Module Organization

- This repo is a Rust workspace (see `Cargo.toml`) with most core crates under `risc0/` (e.g. `risc0/zkvm`, `risc0/zkp`, `risc0/r0vm`).
- `examples/` and `benchmarks/` contain runnable sample projects and perf suites.
- `xtask/` provides repo-specific automation (install helpers, generated artifacts, etc.).
- `tools/` contains developer utilities (some are excluded from the workspace).
- Frontend/docs live in `web/` (Bun + Turbo monorepo) and `website/` (Docusaurus).

## Build, Test, and Development Commands

- LFS prerequisites (needed for some tests/assets): `git lfs install` then `git lfs pull`.
- Rust toolchain is pinned in `rust-toolchain.toml`; use that version for builds/tests.
- Build workspace: `cargo build`.
- Run tests (core): `cargo test`; full CI-like run: `cargo test -F prove -F docker`.
- Format + lint (per `CONTRIBUTING.md`): `cargo fmt --all` and `RISC0_SKIP_BUILD=1 cargo clippy`.
- License audit: `python3 license-check.py`.
- Repo automation: `cargo xtask --help` (e.g. `cargo xtask install`).
- Web app: `cd web && bun install && bun run dev` (also `bun run build|test|check`).
- Docs site: `cd website && bun install && bun run start` (also `bun run build|lint`).

## Coding Style & Naming Conventions

- Rust: `rustfmt` is authoritative; prefer idiomatic Rust 2021 (`snake_case` fns/modules, `UpperCamelCase` types).
- Address `clippy` warnings where practical; avoid introducing new lints in touched code.
- Web: use Biome via `web` scripts (`bun run check`); Website: use `prettier`/`remark` via `website` scripts.

## Testing Guidelines

- Prefer adding/adjusting tests near the change: unit tests in-module, integration tests in `*/tests/*.rs`.
- Some crates use async tests (Tokio) and property tests (`proptest`); follow existing patterns in that crate.

## Commit & Pull Request Guidelines

- Commit messages commonly follow a Conventional-Commits style (e.g. `fix(zkvm): …`, `chore(build): …`); use that format when possible.
- Keep PRs focused: include motivation, test evidence (commands run), and link issues/PRs. Add screenshots for UI/doc rendering changes.
- Before opening a PR, follow the checklist in `CONTRIBUTING.md` (fmt/clippy/tests/license check; update `Cargo.lock` when needed).

## Security & Configuration Tips

- Do not file vulnerabilities as public issues; follow `SECURITY.md` (email `security@risczero.com`).
- When working on proving/toolchain behavior, ensure `cargo-risczero`/`r0vm` versions match the workspace sources.
