type Project = {
  name: string;
  kind: string;
  path: string;
  health: "Clean" | "Changes" | "No Git";
  size: string;
  marker: string;
  color: string;
};

const projects: Project[] = [
  { name: "projectyard", kind: "Rust · Zed", path: "~/Dev/ProjectYard", health: "Clean", size: "14 MB", marker: "Cargo.toml", color: "blue" },
  { name: "paper-trail", kind: "Node.js · VS Code", path: "~/Code/paper-trail", health: "Changes", size: "186 MB", marker: "package.json", color: "amber" },
  { name: "field-notes", kind: "Flutter · Android", path: "~/Work/field-notes", health: "Clean", size: "1.2 GB", marker: "pubspec.yaml", color: "green" },
  { name: "weather-gate", kind: "Python · uv", path: "~/Code/weather-gate", health: "No Git", size: "42 MB", marker: "uv.lock", color: "brick" },
];

export function App() {
  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand"><span className="brand-mark" aria-hidden="true">⌁</span><span>ProjectYard</span></div>
        <nav aria-label="Primary navigation">
          <Section title="Places" items={["Home", "Work drive", "Archive"]} active="Home" />
          <Section title="Filters" items={["All projects", "Technology", "Editor evidence", "Git health"]} active="All projects" />
        </nav>
        <button className="settings">Settings</button>
      </aside>
      <section className="workspace">
        <header className="topbar">
          <div><p className="crumb">Your local inventory</p><h1>Project Yard</h1><p className="summary">143 projects found · last complete scan 4 minutes ago</p></div>
          <button className="scan">Scan locations</button>
        </header>
        <label className="search"><span aria-hidden="true">⌕</span><input placeholder="Search projects, paths, types, and editor evidence" /></label>
        <section className="map-section" aria-labelledby="map-heading">
          <div className="section-heading"><div><p className="crumb">Discovery map</p><h2 id="map-heading">Yard Map</h2></div><p>Area shows local disk use. Colour shows primary technology.</p></div>
          <div className="yard-map" aria-label="Example project storage map">
            <button className="plot plot-work"><strong>Work</strong><span>58 projects</span></button>
            <button className="plot plot-code"><strong>Code</strong><span>44 projects</span></button>
            <button className="plot plot-archive"><strong>Archive</strong><span>41 projects</span></button>
          </div>
        </section>
        <section className="projects" aria-labelledby="projects-heading">
          <div className="section-heading"><div><p className="crumb">Working list</p><h2 id="projects-heading">Projects</h2></div><button className="quiet-button">Sort: recently scanned</button></div>
          <div className="table" role="table" aria-label="Discovered projects">
            <div className="table-row table-head" role="row"><span>Project</span><span>Technology</span><span>Location</span><span>Git</span><span>Size</span></div>
            {projects.map((project) => <ProjectRow key={project.name} project={project} />)}
          </div>
        </section>
      </section>
    </main>
  );
}

function Section({ title, items, active }: { title: string; items: string[]; active: string }) {
  return <div className="nav-section"><p>{title}</p>{items.map((item) => <button className={item === active ? "active" : ""} key={item}>{item}</button>)}</div>;
}

function ProjectRow({ project }: { project: Project }) {
  return <button className="table-row project-row" role="row" title={`Detected by ${project.marker}`}><span className="project-name"><i className={`dot ${project.color}`} />{project.name}</span><span>{project.kind}</span><span className="path">{project.path}</span><span className={`health ${project.health.replace(" ", "-").toLowerCase()}`}>{project.health}</span><span>{project.size}</span></button>;
}
