import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Project = { root: string; kinds: string[]; markers: string[] };
type OpenTarget = "folder" | "zed";
type View = "yard" | "settings";
type ScanMode = "on_demand" | "on_launch";
type SortOrder = "name" | "path" | "technology";
type Inventory = { database: string; projects: Project[]; lastScanUnixSeconds: number | null; directoriesVisited: number | null; issueCount: number | null };
type DiscoverySettings = { scanLocations: string[]; excludedDirectoryNames: string[]; scanMode: ScanMode; scanIntervalMinutes: number };

const preview: Inventory = {
  database: "Preview data — package the desktop app to scan folders",
  lastScanUnixSeconds: null, directoriesVisited: null, issueCount: null,
  projects: [
    { root: "~/Dev/ProjectYard", kinds: ["Rust", "Zed", "Git"], markers: ["Cargo.toml", ".zed", ".git"] },
    { root: "~/Code/paper-trail", kinds: ["Node.js", "VS Code"], markers: ["package.json", ".vscode"] },
    { root: "~/Work/field-notes", kinds: ["Flutter", "Android", "Kotlin"], markers: ["pubspec.yaml", "AndroidManifest.xml"] },
  ],
};
const previewSettings: DiscoverySettings = { scanLocations: ["~/Code"], excludedDirectoryNames: [".cache", "vendor"], scanMode: "on_demand", scanIntervalMinutes: 0 };
const isDesktop = "__TAURI_INTERNALS__" in window;

export function App() {
  const [inventory, setInventory] = useState<Inventory>(preview);
  const [settings, setSettings] = useState<DiscoverySettings>(previewSettings);
  const [view, setView] = useState<View>("yard");
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState("all");
  const [sortOrder, setSortOrder] = useState<SortOrder>("name");
  const [selectedRoot, setSelectedRoot] = useState<string | null>(null);
  const [location, setLocation] = useState("~/Code");
  const [newLocation, setNewLocation] = useState("");
  const [exclusions, setExclusions] = useState(previewSettings.excludedDirectoryNames.join("\n"));
  const [message, setMessage] = useState(isDesktop ? "Reading your local index…" : "Preview mode — the browser never reads your folders.");
  const [settingsMessage, setSettingsMessage] = useState("");
  const [scanning, setScanning] = useState(false);
  const [savingSettings, setSavingSettings] = useState(false);
  const [opening, setOpening] = useState<string | null>(null);
  const scheduledScanInFlight = useRef(false);

  useEffect(() => { if (isDesktop) { void load(); void loadSettings(); } }, []);
  useEffect(() => {
    if (!isDesktop || settings.scanIntervalMinutes === 0 || settings.scanLocations.length === 0) return;
    const interval = window.setInterval(() => {
      if (scheduledScanInFlight.current) return;
      scheduledScanInFlight.current = true;
      setMessage("Refreshing saved locations on schedule…");
      void invoke<Inventory>("scan_saved_locations")
        .then((data) => { setInventory(data); setMessage(`Scheduled refresh found ${data.projects.length} projects.`); })
        .catch((error) => setMessage(String(error)))
        .finally(() => { scheduledScanInFlight.current = false; });
    }, settings.scanIntervalMinutes * 60_000);
    return () => window.clearInterval(interval);
  }, [settings.scanIntervalMinutes, settings.scanLocations.length]);

  const technologyFilters = useMemo(() => kindsFor(inventory.projects, false), [inventory.projects]);
  const editorFilters = useMemo(() => kindsFor(inventory.projects, true), [inventory.projects]);
  const filtered = useMemo(() => sortProjects(inventory.projects.filter((project) => searchable(project).includes(query.trim().toLowerCase()) && matchesFilter(project, filter)), sortOrder), [inventory.projects, query, filter, sortOrder]);
  const groups = useMemo(() => technologyGroups(filtered), [filtered]);
  const selectedProject = filtered.find((project) => project.root === selectedRoot) ?? filtered[0] ?? null;
  const countLabel = `${filtered.length} ${filtered.length === 1 ? "project" : "projects"}`;
  const scanLabel = inventory.lastScanUnixSeconds ? `Indexed ${new Date(inventory.lastScanUnixSeconds * 1000).toLocaleString()}` : "No scan recorded yet";

  async function load() {
    try {
      const data = await invoke<Inventory>("load_inventory");
      setInventory(data);
      setMessage(data.projects.length ? "Local inventory ready." : "Choose a folder to start your yard.");
    } catch (error) { setMessage(`Could not load the local index: ${String(error)}`); }
  }

  async function loadSettings() {
    try {
      const data = await invoke<DiscoverySettings>("load_settings");
      setSettings(data);
      setExclusions(data.excludedDirectoryNames.join("\n"));
    } catch (error) { setSettingsMessage(`Could not load settings: ${String(error)}`); }
  }

  async function scan() {
    if (!location.trim()) return;
    if (!isDesktop) { setMessage("The packaged ProjectYard app scans the folder you enter here. The browser preview does not access your files."); return; }
    setScanning(true); setMessage(`Walking ${location}…`);
    try { const data = await invoke<Inventory>("scan_path", { path: location }); setInventory(data); setMessage(`Added ${data.projects.length} projects from ${location}.`); }
    catch (error) { setMessage(String(error)); }
    finally { setScanning(false); }
  }

  async function scanSavedLocations() {
    if (!isDesktop) { setMessage("The packaged ProjectYard app scans the saved locations. The browser preview does not access your files."); return; }
    setScanning(true); setMessage("Walking saved locations…");
    try { const data = await invoke<Inventory>("scan_saved_locations"); setInventory(data); setMessage(`Refreshed ${data.projects.length} projects from saved locations.`); }
    catch (error) { setMessage(String(error)); }
    finally { setScanning(false); }
  }

  async function openProject(project: Project, target: OpenTarget) {
    if (!isDesktop) { setMessage("Opening projects works in the packaged ProjectYard app. The browser preview does not access your applications."); return; }
    setOpening(`${project.root}:${target}`);
    try { setMessage(await invoke<string>("open_project", { path: project.root, target })); }
    catch (error) { setMessage(String(error)); }
    finally { setOpening(null); }
  }

  function addLocation() {
    const value = newLocation.trim();
    if (!value || settings.scanLocations.includes(value)) return;
    setSettings({ ...settings, scanLocations: [...settings.scanLocations, value] });
    setNewLocation(""); setSettingsMessage("Unsaved location added.");
  }

  async function saveSettings() {
    const next = { ...settings, excludedDirectoryNames: exclusions.split("\n").map((name) => name.trim()).filter(Boolean) };
    if (!isDesktop) { setSettings(next); setSettingsMessage("Preview settings changed locally. The packaged app saves them on this device."); return; }
    setSavingSettings(true); setSettingsMessage("Saving local settings…");
    try { const saved = await invoke<DiscoverySettings>("save_settings", { settings: next }); setSettings(saved); setExclusions(saved.excludedDirectoryNames.join("\n")); setSettingsMessage("Settings saved on this device."); }
    catch (error) { setSettingsMessage(String(error)); }
    finally { setSavingSettings(false); }
  }

  return <main className="app-shell">
    <aside className="sidebar">
      <div className="brand" aria-label="ProjectYard"><span className="brand-mark" aria-hidden="true"><i /><i /><i /></span><span className="brand-type"><b>project</b><em>yard</em></span></div>
      <nav aria-label="ProjectYard navigation"><Section active={view} onSelect={setView} /></nav>
      <div className="rail-bottom"><div className="index-state"><span aria-hidden="true" /><b>Local index</b><small>{isDesktop ? "On this device" : "Preview data"}</small></div></div>
    </aside>
    {view === "yard" ? <Yard inventory={inventory} filtered={filtered} groups={groups} query={query} filter={filter} sortOrder={sortOrder} selectedProject={selectedProject} technologyFilters={technologyFilters} editorFilters={editorFilters} location={location} message={message} scanning={scanning} opening={opening} countLabel={countLabel} scanLabel={scanLabel} savedLocationCount={settings.scanLocations.length} onQuery={setQuery} onFilter={setFilter} onSort={setSortOrder} onSelect={setSelectedRoot} onLocation={setLocation} onScan={scan} onScanSaved={scanSavedLocations} onOpen={openProject} /> : <Settings settings={settings} newLocation={newLocation} exclusions={exclusions} saving={savingSettings} message={settingsMessage} onNewLocation={setNewLocation} onAddLocation={addLocation} onRemoveLocation={(location) => { setSettings({ ...settings, scanLocations: settings.scanLocations.filter((item) => item !== location) }); setSettingsMessage("Unsaved location removed."); }} onMode={(scanMode) => { setSettings({ ...settings, scanMode }); setSettingsMessage("Unsaved scan mode changed."); }} onInterval={(scanIntervalMinutes) => { setSettings({ ...settings, scanIntervalMinutes }); setSettingsMessage("Unsaved scan interval changed."); }} onExclusions={setExclusions} onSave={saveSettings} />}
  </main>;
}

function Section({ active, onSelect }: { active: View; onSelect: (view: View) => void }) {
  return <div className="nav-section"><button className={active === "yard" ? "active" : ""} type="button" onClick={() => onSelect("yard")}>Yard</button><button className={active === "settings" ? "active" : ""} type="button" onClick={() => onSelect("settings")}>Settings</button></div>;
}

function Yard({ inventory, filtered, groups, query, filter, sortOrder, selectedProject, technologyFilters, editorFilters, location, message, scanning, opening, countLabel, scanLabel, savedLocationCount, onQuery, onFilter, onSort, onSelect, onLocation, onScan, onScanSaved, onOpen }: { inventory: Inventory; filtered: Project[]; groups: { name: string; count: number }[]; query: string; filter: string; sortOrder: SortOrder; selectedProject: Project | null; technologyFilters: string[]; editorFilters: string[]; location: string; message: string; scanning: boolean; opening: string | null; countLabel: string; scanLabel: string; savedLocationCount: number; onQuery: (value: string) => void; onFilter: (value: string) => void; onSort: (value: SortOrder) => void; onSelect: (root: string) => void; onLocation: (value: string) => void; onScan: () => void; onScanSaved: () => void; onOpen: (project: Project, target: OpenTarget) => void }) {
  return <section className="workspace">
    <header className="topbar"><div><p className="kicker">Local project inventory</p><h1>The yard</h1></div><div className="head-count"><b>{countLabel}</b><span>{scanLabel}</span></div></header>
    <section className="scan-panel" aria-label="Scan a folder"><div className="scan-intro"><span className="scan-glyph" aria-hidden="true">+</span><p><b>Bring in a folder</b><span>ProjectYard only walks locations you choose.</span></p></div><label className="location-field"><span>Folder</span><input value={location} onChange={(event) => onLocation(event.target.value)} placeholder="~/Code or /mnt/work" /></label><button className="scan" type="button" onClick={onScan} disabled={scanning}>{scanning ? "Scanning…" : "Scan folder"}</button><p className="scan-message" role="status">{message}{savedLocationCount > 0 && <button className="saved-scan" type="button" onClick={onScanSaved} disabled={scanning}>Scan {savedLocationCount} saved {savedLocationCount === 1 ? "location" : "locations"}</button>}</p></section>
    <label className="search"><span>Search</span><input value={query} onChange={(event) => onQuery(event.target.value)} placeholder="Projects, paths, technology, or editor" /></label>
    <section className="map-section" aria-labelledby="map-heading"><div className="section-heading"><div><p className="kicker">A view of what is here</p><h2 id="map-heading">Yard map</h2></div><p>Plots show the technology found in this view. Their size follows project count.</p></div><div className="yard-map" aria-label="Detected projects by technology">{groups.map((group, index) => <div className={`plot plot-${index % 3}`} key={group.name}><span className="plot-number">{String(index + 1).padStart(2, "0")}</span><strong>{group.name}</strong><span>{group.count} {group.count === 1 ? "project" : "projects"}</span></div>)}</div></section>
    <section className="projects" aria-labelledby="projects-heading"><div className="section-heading table-heading"><div><p className="kicker">The register</p><h2 id="projects-heading">Projects</h2></div><p className="index-path">{inventory.database}</p></div><Finder filter={filter} sortOrder={sortOrder} technologyFilters={technologyFilters} editorFilters={editorFilters} onFilter={onFilter} onSort={onSort} />{selectedProject && <ProjectDetail project={selectedProject} opening={opening} onOpen={onOpen} />}<div className="table" role="table" aria-label="Discovered projects"><div className="table-row table-head" role="row"><span>Project</span><span>Technology</span><span>Location</span><span>Found from</span><span>Open</span></div>{filtered.map((project) => <ProjectRow key={project.root} project={project} selected={project.root === selectedProject?.root} opening={opening} onSelect={onSelect} onOpen={onOpen} />)}{!filtered.length && <p className="empty">No projects match this view. Scan a folder or change the search.</p>}</div></section>
  </section>;
}

function Finder({ filter, sortOrder, technologyFilters, editorFilters, onFilter, onSort }: { filter: string; sortOrder: SortOrder; technologyFilters: string[]; editorFilters: string[]; onFilter: (value: string) => void; onSort: (value: SortOrder) => void }) {
  return <div className="finder"><div className="filter-groups"><div className="filter-group"><span>Technology</span><FilterButton active={filter === "all"} onClick={() => onFilter("all")}>All</FilterButton>{technologyFilters.map((kind) => <FilterButton key={kind} active={filter === `kind:${kind}`} onClick={() => onFilter(`kind:${kind}`)}>{kind}</FilterButton>)}</div>{editorFilters.length > 0 && <div className="filter-group"><span>Editor</span>{editorFilters.map((kind) => <FilterButton key={kind} active={filter === `kind:${kind}`} onClick={() => onFilter(`kind:${kind}`)}>{kind}</FilterButton>)}</div>}</div><label className="sort-control"><span>Order</span><select value={sortOrder} onChange={(event) => onSort(event.target.value as SortOrder)}><option value="name">Name</option><option value="path">Path</option><option value="technology">Technology</option></select></label></div>;
}

function FilterButton({ active, onClick, children }: { active: boolean; onClick: () => void; children: string }) {
  return <button className={active ? "filter active" : "filter"} type="button" onClick={onClick}>{children}</button>;
}

function ProjectDetail({ project, opening, onOpen }: { project: Project; opening: string | null; onOpen: (project: Project, target: OpenTarget) => void }) {
  const folderBusy = opening === `${project.root}:folder`; const zedBusy = opening === `${project.root}:zed`;
  return <article className="project-detail"><div className="detail-title"><p>Selected project</p><h3>{projectName(project)}</h3><code>{project.root}</code></div><div className="detail-facts"><div><span>Technology</span><p>{project.kinds.join(" · ")}</p></div><div><span>Found from</span><p>{project.markers.join(", ")}</p></div></div><div className="detail-actions"><button type="button" onClick={() => onOpen(project, "folder")} disabled={Boolean(opening)}>{folderBusy ? "Opening…" : "Open folder"}</button><button type="button" onClick={() => onOpen(project, "zed")} disabled={Boolean(opening)}>{zedBusy ? "Opening…" : "Open in Zed"}</button></div></article>;
}

function Settings({ settings, newLocation, exclusions, saving, message, onNewLocation, onAddLocation, onRemoveLocation, onMode, onInterval, onExclusions, onSave }: { settings: DiscoverySettings; newLocation: string; exclusions: string; saving: boolean; message: string; onNewLocation: (value: string) => void; onAddLocation: () => void; onRemoveLocation: (location: string) => void; onMode: (mode: ScanMode) => void; onInterval: (minutes: number) => void; onExclusions: (value: string) => void; onSave: () => void }) {
  return <section className="workspace settings-workspace"><header className="topbar"><div><p className="kicker">Local discovery controls</p><h1>Settings</h1></div><p className="settings-head-note">Nothing leaves this device.</p></header><div className="settings-sheet"><section className="settings-section"><div><h2>Scan locations</h2><p>These are the only places ProjectYard will scan when you choose saved locations.</p></div><div className="location-add"><input value={newLocation} onChange={(event) => onNewLocation(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") onAddLocation(); }} placeholder="~/Code or /mnt/work" aria-label="New scan location" /><button type="button" onClick={onAddLocation}>Add location</button></div><ul className="location-list">{settings.scanLocations.map((location) => <li key={location}><code>{location}</code><button type="button" onClick={() => onRemoveLocation(location)}>Remove</button></li>)}{!settings.scanLocations.length && <li className="empty-location">No saved locations yet.</li>}</ul></section><section className="settings-section"><div><h2>When to scan</h2><p>On-demand scanning stays in your control. Launch scanning refreshes saved locations whenever ProjectYard opens.</p></div><div className="mode-options"><label><input type="radio" name="scan-mode" checked={settings.scanMode === "on_demand"} onChange={() => onMode("on_demand")} /><span><b>On demand</b><small>Only scan after you choose a scan action.</small></span></label><label><input type="radio" name="scan-mode" checked={settings.scanMode === "on_launch"} onChange={() => onMode("on_launch")} /><span><b>On app launch</b><small>Refresh all saved locations when ProjectYard opens.</small></span></label></div></section><section className="settings-section"><div><h2>While the app is open</h2><p>Optionally refresh saved locations on a fixed interval. This stops when ProjectYard closes.</p></div><div className="interval-control"><span id="interval-label">Refresh interval</span><div className="interval-options" role="group" aria-labelledby="interval-label">{([{ minutes: 0, label: "Off" }, { minutes: 15, label: "15 min" }, { minutes: 30, label: "30 min" }, { minutes: 60, label: "Hourly" }] as const).map((option) => <button className={settings.scanIntervalMinutes === option.minutes ? "active" : ""} type="button" key={option.minutes} onClick={() => onInterval(option.minutes)} aria-pressed={settings.scanIntervalMinutes === option.minutes}>{option.label}</button>)}</div><small>Scheduled refreshes only scan saved locations.</small></div></section><section className="settings-section"><div><h2>Folder-name exclusions</h2><p>One folder name per line. These are added to ProjectYard’s built-in exclusions, such as <code>node_modules</code> and <code>target</code>.</p></div><textarea value={exclusions} onChange={(event) => onExclusions(event.target.value)} placeholder=".cache&#10;vendor" aria-label="Excluded folder names" /></section><footer className="settings-footer"><p role="status">{message || "Changes remain local until you save them."}</p><button className="scan save-settings" type="button" onClick={onSave} disabled={saving}>{saving ? "Saving…" : "Save settings"}</button></footer></div></section>;
}

function ProjectRow({ project, selected, opening, onSelect, onOpen }: { project: Project; selected: boolean; opening: string | null; onSelect: (root: string) => void; onOpen: (project: Project, target: OpenTarget) => void }) {
  const name = projectName(project);
  const folderBusy = opening === `${project.root}:folder`; const zedBusy = opening === `${project.root}:zed`;
  return <div className={selected ? "table-row project-row selected" : "table-row project-row"} role="row" title={`Detected by ${project.markers.join(", ")}`}><button className="project-select" type="button" onClick={() => onSelect(project.root)} aria-pressed={selected}><i className="dot" />{name}</button><span>{project.kinds.join(" · ")}</span><span className="path">{project.root}</span><span className="marker">{project.markers.join(", ")}</span><span className="project-actions"><button type="button" onClick={() => onOpen(project, "folder")} disabled={Boolean(opening)}>{folderBusy ? "Opening…" : "Folder"}</button><button type="button" onClick={() => onOpen(project, "zed")} disabled={Boolean(opening)}>{zedBusy ? "Opening…" : "Zed"}</button></span></div>;
}

function searchable(project: Project) { return `${project.root} ${project.kinds.join(" ")} ${project.markers.join(" ")}`.toLowerCase(); }
function technologyGroups(projects: Project[]) { const counts = new Map<string, number>(); projects.forEach((project) => { const primary = project.kinds[0] || "Other"; counts.set(primary, (counts.get(primary) || 0) + 1); }); const groups = [...counts].map(([name, count]) => ({ name, count })).sort((left, right) => right.count - left.count).slice(0, 3); return groups.length ? groups : [{ name: "No projects yet", count: 0 }]; }
function projectName(project: Project) { return project.root.split("/").filter(Boolean).at(-1) || project.root; }
function isEditor(kind: string) { return ["Zed", "VS Code", "JetBrains", "Fleet", "Cursor"].includes(kind); }
function kindsFor(projects: Project[], editors: boolean) { return [...new Set(projects.flatMap((project) => project.kinds).filter((kind) => isEditor(kind) === editors && kind !== "Git"))].sort((left, right) => left.localeCompare(right)); }
function matchesFilter(project: Project, filter: string) { return filter === "all" || project.kinds.includes(filter.replace("kind:", "")); }
function sortProjects(projects: Project[], sortOrder: SortOrder) { return [...projects].sort((left, right) => { const value = sortOrder === "path" ? left.root.localeCompare(right.root) : sortOrder === "technology" ? (left.kinds[0] || "").localeCompare(right.kinds[0] || "") : projectName(left).localeCompare(projectName(right)); return value || left.root.localeCompare(right.root); }); }
