# ProjectYard roadmap

This roadmap is the source of truth for product progress. Each release is a
usable version category rather than an open-ended collection of features.

Status keys:

- **Done** — implemented and verified.
- **In progress** — actively being built for the next release.
- **Planned** — agreed direction; not yet started.
- **Later** — valuable, but intentionally outside the current release focus.

## v0.1 — Local foundation

**Goal:** reliably discover local projects on Ubuntu and establish one shared
Rust core for the GUI and TUI.

- **Done** — Rust workspace with scanner, CLI, TUI shell, and desktop shell.
- **Done** — Evidence-based detection for Cargo, Node.js, Python, uv, Flutter,
  Gradle, Android, Kotlin, Maven, Go, Ruby, PHP, Xcode, Git, and editor
  footprints.
- **Done** — Default exclusions for dependency, build, cache, and VCS folders.
- **Done** — Local SQLite index with atomic scan replacement and scan history.
- **Done** — MIT licensing and no-telemetry, local-first product contract.
- **In progress** — Connect real scanner/index data to the desktop UI.
- **Done** — Interactive TUI with keyboard navigation, search, reload, and
  evidence details from the shared SQLite index.

**Exit criteria:** a user can index selected Ubuntu folders, inspect the same
stored projects through a real GUI or TUI, see why each project was detected,
and open its location.

## v0.2 — Discovery workflow

**Goal:** make the first-run and recurring discovery experience useful without
requiring a terminal.

- **Planned** — Tauri desktop integration for the existing React shell.
- **Planned** — Select scan locations and manage exclusion rules in settings.
- **Planned** — On-demand scanning, launch-time scanning, and in-app interval
  scanning.
- **Planned** — Search, filters, sorting, and project detail panels backed by
  the SQLite index.
- **Planned** — Open a project in its folder or a configured editor; Zed is the
  initial default.
- **Planned** — Real Yard Map: disk-use treemap grouped by scan location and
  coloured by primary technology.

**Exit criteria:** discovery, finding, filtering, and opening a local project
work entirely from the GUI.

## v0.3 — Terminal-first companion

**Goal:** give terminal users a fast, comfortable alternative to the desktop
interface—not a wall of CLI output.

- **Planned** — Ratatui-based interactive project browser.
- **Planned** — Keyboard search, filters, selection, details, and open actions.
- **Planned** — Shared database and configuration with the GUI.
- **Planned** — Small supporting CLI surface for `scan`, `index`, `config`,
  `export`, `gui`, and `tui`.

**Exit criteria:** GUI and TUI show the same inventory and can be used
independently for normal discovery workflows.

## v0.4 — Project health and storage

**Goal:** add the most useful second-order signals after discovery is solid.

- **Planned** — Disk-use calculation that separates project source from ignored
  dependency/build output.
- **Planned** — Git health: repository state, dirty files, branch, remotes, and
  last commit time.
- **Planned** — Dependency-summary adapters, starting with Cargo, Node.js,
  Python/uv, Flutter, and Gradle.
- **Planned** — Safe duplicate and nested-project hints.

**Exit criteria:** users can decide what to revisit, clean up, or open from a
single project record.

## v0.5 — Reliable background behaviour

**Goal:** let users choose how fresh their inventory stays while preserving
privacy and control.

- **Planned** — Configurable launch-time and fixed-interval scans while the app
  is open.
- **Planned** — Optional selected-folder filesystem watching.
- **Later** — Opt-in Ubuntu user service for scans while the app is closed.
- **Planned** — Scan history, errors, permissions, and exclusion explanations.

**Exit criteria:** scheduled or watched scans are transparent, reversible, and
never scan locations the user has not chosen.

## v1.0 — Stable personal project yard

**Goal:** a polished Ubuntu-first release that people can rely on every day.

- **Planned** — Robust packaging: `.deb` and AppImage.
- **Planned** — Light, dark, and system theme choices with accessible contrast.
- **Planned** — Import/export of settings and a documented database location.
- **Planned** — Performance and accessibility pass on large, mixed filesystems.
- **Later** — Windows and macOS packaging, using the shared Rust core.

**Exit criteria:** stable release documentation, migration notes, automated
verification, and a dependable Ubuntu distribution.

## Update policy

Every shipped change should update this file in the same commit: move completed
items to **Done**, add scope that enters the next version, and record any
intentional deferrals under **Later**. This keeps the roadmap useful without
turning it into a promise of fixed dates.
