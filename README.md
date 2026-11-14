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
cargo run -p projectyard-tui -- --database .projectyard/index.sqlite
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

The TUI reads that same index. Use `/` to search, `j`/`k` or arrow keys to
select projects, `r` to reload the database, and `q` to quit.

## Desktop development on Ubuntu

The desktop app is a Tauri shell around the same scanner and SQLite index. Its
scan control accepts one folder path at a time, so it never begins a broad scan
without the user choosing a location. The development machine needs the usual
Tauri WebKitGTK packages before building the native app:

```sh
sudo apt-get install libwebkit2gtk-4.1-dev libsoup-3.0-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf
```

Then run the desktop shell from `apps/desktop` with the Tauri CLI. The browser
preview intentionally uses labeled sample data and cannot read local folders.

## License

[MIT](LICENSE)
