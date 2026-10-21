# ProjectYard design direction

## Job and audience

ProjectYard helps one developer regain an accurate, navigable picture of the
projects on their computer. Discovery is the first job; disk use, opening a
project, Git health, and dependencies follow that order.

## Visual system

ProjectYard is a dark-first cartographer's workbench rather than a generic
developer dashboard. The Yard Map is the signature view: a treemap of local
project locations where area represents disk use and colour represents the
primary detected ecosystem. It always works with a readable project list,
rather than replacing it.

| Name | Value | Use |
| --- | --- | --- |
| Deep yard | `#101C23` | App canvas |
| Slate surface | `#172A34` | Sidebar and stable surfaces |
| Raised slate | `#213A46` | Inputs, focused rows, popovers |
| Chalk | `#E8F0F2` | Main text |
| Mist | `#9CB1B9` | Secondary text |
| Harbor | `#4AA4C6` | Primary action and selection |
| Moss | `#61B68D` | Healthy state |
| Amber | `#D5A646` | Attention state |
| Brick | `#D06A5A` | Error state |

IBM Plex Sans is the UI face; IBM Plex Mono is limited to paths, manifest
names, commands, and hashes. The user can choose Light or System themes; the
dark palette is the default.

## Layout

```text
┌───────────────┬─────────────────────────────────────────────────────┐
│ ProjectYard   │ Project Yard                              Scan now   │
│               │ 143 projects · last scanned 4 minutes ago           │
│ Places        ├─────────────────────────────────────────────────────┤
│   Home        │ Search projects, paths, types, and editor evidence  │
│   Work drive  ├─────────────────────────────────────────────────────┤
│               │ Yard Map                                            │
│ Filters       │ location groups with ecosystem markers               │
│   Type        ├─────────────────────────────────────────────────────┤
│   Editor      │ Projects                                             │
│   Git health  │ Name       Type       Path         Git      Size     │
│               │ projectyard Rust       ~/Dev/...    Clean    14 MB   │
│ Settings      │                                                     │
└───────────────┴─────────────────────────────────────────────────────┘
```

The list is left-aligned and the default working surface. Oversized cards,
unexplained scores, and decorative animation are deliberately excluded.
Every detected tag can reveal the marker that produced it.

## Accessibility and behaviour

- Keyboard navigation and visible focus are required in both frontends.
- Scans stream progress and partial results rather than blocking the screen.
- Reduced-motion preferences are respected.
- Exclusions are visible and explainable.
- Copy uses direct actions: “Scan locations”, “Open in Zed”, and “Exclude
  folder”.

## Data boundaries

The shared Rust scanner produces evidence-based project records. The local
SQLite index replaces the active inventory in one transaction, then retains a
small scan-run history for “last scanned” and diagnostics. The GUI and TUI will
read this same index; neither needs a separate project cache. No scan metadata
is transmitted or collected as telemetry.
