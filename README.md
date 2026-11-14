# ProjectYard

ProjectYard is a local-first project inventory for developers. It discovers
projects across configured folders, explains how every project was detected,
and presents the same index through a desktop GUI or terminal UI.

ProjectYard is private by default: scans, metadata, and future SQLite indexes
remain on the machine. It contains no telemetry.

## Status

ProjectYard is in **v0.2 — Discovery workflow**. The local foundation is
complete; persistent discovery settings and saved-location scanning are now in
place. See the versioned [roadmap](ROADMAP.md) for completed work, release
criteria, and future scope.

## Interfaces

- **Desktop GUI** — an Ubuntu-first native application for scanning a selected
  folder, viewing its local inventory, and seeing the Yard Map.
- **TUI** — a fast terminal view over an explicit local SQLite inventory, with
  search, selection, reload, and open controls.
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

The TUI reads the database path passed with `--database`. Use `/` to search,
`j`/`k` or arrow keys to select projects, `o` to open the selected project
folder, `z` to open it in Zed, `r` to reload the database, and `q` to quit.

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

Each indexed desktop project has a **Folder** action for the Ubuntu file
manager and a **Zed** action. The latter uses Zed's `zed` command-line launcher
and explains how to proceed if that launcher is unavailable.

## Discovery settings

The desktop **Settings** view stores discovery preferences in the same local
SQLite database as the desktop inventory. Add the folders you want ProjectYard
to scan, add extra folder-name exclusions, and choose either **On demand**
(the default) or **On app launch** scanning. You can also choose a 15-, 30-,
or 60-minute refresh interval while the desktop app is open. Use **Scan saved
locations** from the Yard to refresh all saved folders explicitly.

ProjectYard never scans a location that has not been added or entered by you.
Interval scanning stops when the app closes.

## Finding projects

Use the Yard search field for project names, paths, technologies, and editor
evidence. The register also has separate technology and editor filters and can
be ordered by name, path, or primary technology. Select a project name to see
its location, all detected kinds, and the markers that caused ProjectYard to
include it.

To create the Ubuntu installer after a successful native build:

```sh
npm run package:ubuntu
```

The generated `.deb` is placed under `target/release/bundle/deb/` at the
workspace root. The optimized native binary is placed at
`target/release/projectyard-desktop`.

## License

[MIT](LICENSE)
