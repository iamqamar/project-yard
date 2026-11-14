import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Project = { root: string; kinds: string[]; markers: string[] };
type Inventory = {
  database: string;
  projects: Project[];
  lastScanUnixSeconds: number | null;
  directoriesVisited: number | null;
  issueCount: number | null;
};

const preview: Inventory = {
  database: "Preview data — package the desktop app to scan folders",
  lastScanUnixSeconds: null,
  directoriesVisited: null,
  issueCount: null,
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
  const [message, setMessage] = useState(
    isDesktop ? "Reading your local index…" : "Preview mode — the browser never reads your folders.",
  );
  const [scanning, setScanning] = useState(false);

  useEffect(() => { if (isDesktop) void load(); }, []);

  const filtered = useMemo(
    () => inventory.projects.filter((project) => searchable(project).includes(query.trim().toLowerCase())),
    [inventory.projects, query],
  );
  const groups = useMemo(() => technologyGroups(filtered), [filtered]);
  const countLabel = `${filtered.length} ${filtered.length === 1 ? "project" : "projects"}`;
  const scanLabel = inventory.lastScanUnixSeconds
    ? `Indexed ${new Date(inventory.lastScanUnixSeconds * 1000).toLocaleString()}`
    : "No scan recorded yet";

  async function load() {
    try {
      const data = await invoke<Inventory>("load_inventory");
      setInventory(data);
      setMessage(data.projects.length ? "Local inventory ready." : "Choose a folder to start your yard.");
    } catch (error) { setMessage(`Could not load the local index: ${String(error)}`); }
  }

  async function scan() {
    if (!location.trim()) return;
    if (!isDesktop) {
      setMessage("The packaged ProjectYard app scans the folder you enter here. The browser preview does not access your files.");
      return;
    }
    setScanning(true);
    setMessage(`Walking ${location}…`);
    try {
      const data = await invoke<Inventory>("scan_path", { path: location });
      setInventory(data);
      setMessage(`Added ${data.projects.length} projects from ${location}.`);
    } catch (error) { setMessage(String(error)); }
    finally { setScanning(false); }
  }

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand" aria-label="ProjectYard">
          <span className="brand-mark" aria-hidden="true"><i /><i /><i /></span>
          <span className="brand-type"><b>project</b><em>yard</em></span>
        </div>
        <nav aria-label="Inventory navigation"><Section items={["Yard", "Evidence"]} active="Yard" /></nav>
        <div className="rail-bottom">
          <div className="index-state"><span aria-hidden="true" /><b>Local index</b><small>{isDesktop ? "On this device" : "Preview data"}</small></div>
          <button className="settings" type="button">Settings</button>
        </div>
      </aside>

      <section className="workspace">
        <header className="topbar">
          <div><p className="kicker">Local project inventory</p><h1>The yard</h1></div>
          <div className="head-count"><b>{countLabel}</b><span>{scanLabel}</span></div>
        </header>

        <section className="scan-panel" aria-label="Scan a folder">
          <div className="scan-intro"><span className="scan-glyph" aria-hidden="true">+</span><p><b>Bring in a folder</b><span>ProjectYard only walks locations you choose.</span></p></div>
          <label className="location-field"><span>Folder</span><input value={location} onChange={(event) => setLocation(event.target.value)} placeholder="~/Code or /mnt/work" /></label>
          <button className="scan" type="button" onClick={scan} disabled={scanning}>{scanning ? "Scanning…" : "Scan"}</button>
          <p className="scan-message" role="status">{message}</p>
        </section>

        <label className="search"><span>Search</span><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Projects, paths, technology, or editor" /></label>

        <section className="map-section" aria-labelledby="map-heading">
          <div className="section-heading"><div><p className="kicker">A view of what is here</p><h2 id="map-heading">Yard map</h2></div><p>Plots show the technology found in this view. Their size follows project count.</p></div>
          <div className="yard-map" aria-label="Detected projects by technology">
            {groups.map((group, index) => <div className={`plot plot-${index % 3}`} key={group.name}><span className="plot-number">{String(index + 1).padStart(2, "0")}</span><strong>{group.name}</strong><span>{group.count} {group.count === 1 ? "project" : "projects"}</span></div>)}
          </div>
        </section>

        <section className="projects" aria-labelledby="projects-heading">
          <div className="section-heading table-heading"><div><p className="kicker">The register</p><h2 id="projects-heading">Projects</h2></div><p className="index-path">{inventory.database}</p></div>
          <div className="table" role="table" aria-label="Discovered projects">
            <div className="table-row table-head" role="row"><span>Project</span><span>Technology</span><span>Location</span><span>Found from</span></div>
            {filtered.map((project) => <ProjectRow key={project.root} project={project} />)}
            {!filtered.length && <p className="empty">No projects match this view. Scan a folder or change the search.</p>}
          </div>
        </section>
      </section>
    </main>
  );
}

function Section({ items, active }: { items: string[]; active: string }) {
  return <div className="nav-section">{items.map((item) => <button className={item === active ? "active" : ""} type="button" key={item}>{item}</button>)}</div>;
}

function ProjectRow({ project }: { project: Project }) {
  const name = project.root.split("/").filter(Boolean).at(-1) || project.root;
  return <button className="table-row project-row" role="row" type="button" title={`Detected by ${project.markers.join(", ")}`}><span className="project-name"><i className="dot" />{name}</span><span>{project.kinds.join(" · ")}</span><span className="path">{project.root}</span><span className="marker">{project.markers.join(", ")}</span></button>;
}

function searchable(project: Project) { return `${project.root} ${project.kinds.join(" ")} ${project.markers.join(" ")}`.toLowerCase(); }
function technologyGroups(projects: Project[]) {
  const counts = new Map<string, number>();
  projects.forEach((project) => { const primary = project.kinds[0] || "Other"; counts.set(primary, (counts.get(primary) || 0) + 1); });
  const groups = [...counts].map(([name, count]) => ({ name, count })).sort((left, right) => right.count - left.count).slice(0, 3);
  return groups.length ? groups : [{ name: "No projects yet", count: 0 }];
}
