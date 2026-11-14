import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Project = { root: string; kinds: string[]; markers: string[] };
type Inventory = { database: string; projects: Project[]; lastScanUnixSeconds: number | null; directoriesVisited: number | null; issueCount: number | null };

const preview: Inventory = {
  database: "Preview data — package the desktop app to scan folders",
  lastScanUnixSeconds: null, directoriesVisited: null, issueCount: null,
  projects: [
    { root: "~/Dev/ProjectYard", kinds: ["Rust", "Zed", "Git"], markers: ["Cargo.toml", ".zed", ".git"] },
    { root: "~/Code/paper-trail", kinds: ["Node.js", "VS Code"], markers: ["package.json", ".vscode"] },
    { root: "~/Work/field-notes", kinds: ["Flutter", "Android", "Kotlin"], markers: ["pubspec.yaml", "AndroidManifest.xml"] },
  ],
};
const isDesktop = "__TAURI_INTERNALS__" in window;

export function App() {
  const [inventory, setInventory] = useState<Inventory>(preview);
  const [query, setQuery] = useState("");
  const [location, setLocation] = useState("~/Code");
  const [message, setMessage] = useState(isDesktop ? "Loading local inventory…" : "Preview mode — no folders are read in the browser.");
  const [scanning, setScanning] = useState(false);
  useEffect(() => { if (isDesktop) void load(); }, []);
  const filtered = useMemo(() => inventory.projects.filter((project) => searchable(project).includes(query.trim().toLowerCase())), [inventory.projects, query]);
  const groups = useMemo(() => technologyGroups(filtered), [filtered]);

  async function load() {
    try { const data = await invoke<Inventory>("load_inventory"); setInventory(data); setMessage(data.projects.length ? "Local inventory loaded." : "Choose a folder to create your first local inventory."); }
    catch (error) { setMessage(`Could not load the local index: ${String(error)}`); }
  }
  async function scan() {
    if (!location.trim()) return;
    if (!isDesktop) { setMessage("The packaged ProjectYard app scans the folder you enter here. The browser preview does not access your files."); return; }
    setScanning(true); setMessage(`Scanning ${location}…`);
    try { const data = await invoke<Inventory>("scan_path", { path: location }); setInventory(data); setMessage(`Indexed ${data.projects.length} projects from ${location}.`); }
    catch (error) { setMessage(String(error)); }
    finally { setScanning(false); }
  }
  const summary = inventory.lastScanUnixSeconds ? `${filtered.length} matching projects · scanned ${new Date(inventory.lastScanUnixSeconds * 1000).toLocaleString()}` : `${filtered.length} ${filtered.length === 1 ? "project" : "projects"} visible`;
  return <main className="app-shell">
    <aside className="sidebar"><div className="brand"><span className="brand-mark" aria-hidden="true">⌁</span><span>ProjectYard</span></div><nav aria-label="Inventory navigation"><Section title="Inventory" items={["All projects", "Technology map", "Detection evidence"]} active="All projects" /><Section title="Coming next" items={["Disk usage", "Git health", "Dependencies"]} active="" /></nav><p className="local-note">Everything stays on this device.</p><button className="settings" type="button">Settings</button></aside>
    <section className="workspace">
      <header className="topbar"><div><p className="crumb">Your local inventory</p><h1>Project Yard</h1><p className="summary">{summary}</p></div></header>
      <section className="scan-panel" aria-label="Scan a folder"><label><span>Folder to scan</span><input value={location} onChange={(event) => setLocation(event.target.value)} placeholder="~/Code or /mnt/work" /></label><button className="scan" type="button" onClick={scan} disabled={scanning}>{scanning ? "Scanning…" : "Scan folder"}</button><p className="scan-message" role="status">{message}</p></section>
      <label className="search"><span aria-hidden="true">⌕</span><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search projects, paths, types, and editor evidence" /></label>
      <section className="map-section" aria-labelledby="map-heading"><div className="section-heading"><div><p className="crumb">Discovery map</p><h2 id="map-heading">Technology map</h2></div><p>Tile area represents detected project count. Disk use arrives with the next milestone.</p></div><div className="yard-map" aria-label="Detected projects by technology">{groups.map((group, index) => <div className={`plot plot-${index % 3}`} key={group.name}><strong>{group.name}</strong><span>{group.count} {group.count === 1 ? "project" : "projects"}</span></div>)}</div></section>
      <section className="projects" aria-labelledby="projects-heading"><div className="section-heading"><div><p className="crumb">Working list</p><h2 id="projects-heading">Projects</h2></div><p className="index-path">{inventory.database}</p></div><div className="table" role="table" aria-label="Discovered projects"><div className="table-row table-head" role="row"><span>Project</span><span>Technology</span><span>Location</span><span>Detected by</span></div>{filtered.map((project) => <ProjectRow key={project.root} project={project} />)}{!filtered.length && <p className="empty">No projects match this view. Scan a folder or change the search.</p>}</div></section>
    </section>
  </main>;
}
function Section({ title, items, active }: { title: string; items: string[]; active: string }) { return <div className="nav-section"><p>{title}</p>{items.map((item) => <button className={item === active ? "active" : ""} type="button" key={item}>{item}</button>)}</div>; }
function ProjectRow({ project }: { project: Project }) { const name = project.root.split("/").filter(Boolean).at(-1) || project.root; return <button className="table-row project-row" role="row" type="button" title={`Detected by ${project.markers.join(", ")}`}><span className="project-name"><i className="dot blue" />{name}</span><span>{project.kinds.join(" · ")}</span><span className="path">{project.root}</span><span className="marker">{project.markers.join(", ")}</span></button>; }
function searchable(project: Project) { return `${project.root} ${project.kinds.join(" ")} ${project.markers.join(" ")}`.toLowerCase(); }
function technologyGroups(projects: Project[]) { const counts = new Map<string, number>(); projects.forEach((project) => { const primary = project.kinds[0] || "Other"; counts.set(primary, (counts.get(primary) || 0) + 1); }); const groups = [...counts].map(([name, count]) => ({ name, count })).sort((left, right) => right.count - left.count).slice(0, 3); return groups.length ? groups : [{ name: "No projects yet", count: 0 }]; }
