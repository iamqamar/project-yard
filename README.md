# ProjectYard

ProjectYard is a local-first project inventory for developers. It discovers
projects across configured folders, explains how every project was detected,
and presents the same index through a desktop GUI or terminal UI.

ProjectYard is private by default: scans, metadata, and future SQLite indexes
remain on the machine. It contains no telemetry.

## Status

ProjectYard is in **v0.1 — Local foundation**. The scanner, local SQLite index,
desktop inventory, and interactive TUI are working. See the versioned
[roadmap](ROADMAP.md) for completed work, release criteria, and future scope.

## Interfaces

- **Desktop GUI** — an Ubuntu-first native application for scanning a selected
  folder, viewing its local inventory, and seeing the Yard Map.
- **TUI** — a fast terminal view over the same SQLite inventory, with search,
  selection, and reload controls.
- **Supporting CLI** — focused commands for scanning, detecting a directory,
  and creating or refreshing an index. It is intentionally not the primary
  data-browsing interface.

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
location is explicit during the early development phase. The desktop app stores
its own inventory in its per-user application-data directory.

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

Then install the JavaScript packages and run the native shell from
`apps/desktop`:

```sh
npm install
npm run desktop
```

Use the scan field to choose a folder, such as `~/Code`. ProjectYard scans only
that explicit location; it never starts a whole-disk scan automatically. The
browser preview (`npm run dev`) intentionally uses labeled sample data and
cannot read local folders.

To create the Ubuntu installer after a successful native build:

```sh
npm run package:ubuntu
```

The generated `.deb` is placed under `target/release/bundle/deb/` at the
workspace root.

## License

[MIT](LICENSE)
