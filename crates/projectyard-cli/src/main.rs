use std::{env, path::PathBuf, process::ExitCode};

use projectyard_core::{Project, ScanConfig, Scanner, detect_project};
use projectyard_index::ProjectIndex;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("ProjectYard: {message}");
            eprintln!("Run `projectyard help` for usage.");
            ExitCode::from(2)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let command = args.first().map(String::as_str).unwrap_or("help");
    match command {
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "scan" => scan(&args[1..]),
        "index" => index(&args[1..]),
        "detect" => detect(&args[1..]),
        "tui" => Err("The TUI is currently a separate `projectyard-tui` binary.".to_owned()),
        "gui" => {
            Err("The desktop shell is being wired up; use `projectyard-tui` for now.".to_owned())
        }
        other => Err(format!("unknown command `{other}`")),
    }
}

fn index(args: &[String]) -> Result<(), String> {
    let (database, roots) = parse_index_arguments(args)?;
    let result = Scanner::new(ScanConfig::for_roots(roots)).scan();
    let mut index = ProjectIndex::open(&database).map_err(|error| error.to_string())?;
    let summary = index
        .save_scan(&result)
        .map_err(|error| error.to_string())?;
    println!(
        "Indexed {} projects in {}. Visited {} folders; skipped {} unreadable locations.",
        summary.project_count,
        database.display(),
        summary.directories_visited,
        summary.issue_count,
    );
    Ok(())
}

fn parse_index_arguments(args: &[String]) -> Result<(PathBuf, Vec<PathBuf>), String> {
    let mut database = PathBuf::from(".projectyard/index.sqlite");
    let mut roots = Vec::new();
    let mut position = 0;
    while position < args.len() {
        if args[position] == "--database" {
            position += 1;
            let value = args
                .get(position)
                .ok_or("--database requires a file path")?;
            database = PathBuf::from(value);
        } else {
            roots.push(PathBuf::from(&args[position]));
        }
        position += 1;
    }
    if roots.is_empty() {
        roots.push(env::current_dir().map_err(|error| error.to_string())?);
    }
    Ok((database, roots))
}

fn scan(args: &[String]) -> Result<(), String> {
    let json = args.iter().any(|argument| argument == "--json");
    let roots: Vec<PathBuf> = args
        .iter()
        .filter(|argument| argument.as_str() != "--json")
        .map(PathBuf::from)
        .collect();
    let roots = if roots.is_empty() {
        vec![env::current_dir().map_err(|error| error.to_string())?]
    } else {
        roots
    };
    let result = Scanner::new(ScanConfig::for_roots(roots)).scan();
    if json {
        print_json(&result.projects);
    } else {
        println!(
            "Found {} projects after visiting {} folders.\n",
            result.projects.len(),
            result.directories_visited
        );
        for project in &result.projects {
            print_project(project);
        }
        if !result.issues.is_empty() {
            println!("Skipped {} unreadable locations.", result.issues.len());
        }
    }
    Ok(())
}

fn detect(args: &[String]) -> Result<(), String> {
    let path = args.first().ok_or("detect requires a project directory")?;
    match detect_project(&PathBuf::from(path)) {
        Some(project) => {
            print_project(&project);
            Ok(())
        }
        None => Err(format!("no recognized markers in {path}")),
    }
}

fn print_project(project: &Project) {
    let kinds = project
        .kinds
        .iter()
        .map(|kind| kind.label())
        .collect::<Vec<_>>()
        .join(", ");
    let evidence = project
        .evidence
        .iter()
        .map(|item| item.marker.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    println!(
        "{}\n  {kinds}\n  detected by: {evidence}\n",
        project.root.display()
    );
}

fn print_json(projects: &[Project]) {
    println!("[");
    for (index, project) in projects.iter().enumerate() {
        let kinds = project
            .kinds
            .iter()
            .map(|kind| format!("\"{}\"", kind.label()))
            .collect::<Vec<_>>()
            .join(", ");
        let evidence = project
            .evidence
            .iter()
            .map(|item| format!("\"{}\"", escape(&item.marker)))
            .collect::<Vec<_>>()
            .join(", ");
        let suffix = if index + 1 == projects.len() { "" } else { "," };
        println!(
            "  {{\"path\":\"{}\",\"kinds\":[{kinds}],\"evidence\":[{evidence}]}}{suffix}",
            escape(&project.root.display().to_string())
        );
    }
    println!("]");
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn print_help() {
    println!(
        "ProjectYard — local project discovery\n\nUsage:\n  projectyard scan [PATH ...] [--json]\n  projectyard index [PATH ...] [--database FILE]\n  projectyard detect PATH\n\nThe interactive terminal interface is available as `projectyard-tui`.\nAll scans are local; ProjectYard sends no telemetry."
    );
}
