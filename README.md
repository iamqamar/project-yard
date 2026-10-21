# ProjectYard

ProjectYard is a local-first project inventory for developers. It discovers
projects across configured folders, explains how every project was detected,
and presents the same index through a desktop GUI or terminal UI.

ProjectYard is private by default: scans, metadata, and future SQLite indexes
remain on the machine. It contains no telemetry.

## Status

ProjectYard is in **v0.1 — Local foundation**. The Rust scanner and local
SQLite index are working; connecting the stored inventory to the desktop UI and
an interactive TUI is the current focus. See the versioned [roadmap](ROADMAP.md)
for completed work, release criteria, and future scope.

## Current commands

```sh
cargo run -p projectyard-cli -- scan ~/Code
cargo run -p projectyard-cli -- scan ~/Code --json
cargo run -p projectyard-cli -- index ~/Code
cargo run -p projectyard-cli -- index ~/Code --database /path/to/index.sqlite
cargo run -p projectyard-cli -- detect /path/to/project
cargo run -p projectyard-tui -- ~/Code
```

## Detection support

The scanner currently recognizes Rust/Cargo, Node.js, Python, uv, Flutter,
Gradle, Android, Kotlin, Maven, Go, Ruby, PHP, Xcode, Git, Zed, VS Code,
JetBrains IDEs, Fleet, and Cursor evidence. A project may have several tags;
ProjectYard never forces one ecosystem label.

`projectyard index` stores a scan in a local SQLite database. It defaults to
`.projectyard/index.sqlite` beneath the current directory so the storage
location is explicit during the early development phase. The desktop app will
later own the standard per-user database location and scan settings.

## License

[MIT](LICENSE)
