//! Terminal view scaffold. The presentation intentionally mirrors the GUI's
//! language while the interactive Ratatui layer is introduced next.

use std::{env, path::PathBuf};

use projectyard_core::{Project, ScanConfig, Scanner};

fn main() {
    let roots: Vec<PathBuf> = env::args().skip(1).map(PathBuf::from).collect();
    let roots = if roots.is_empty() {
        vec![env::current_dir().expect("current directory")]
    } else {
        roots
    };
    let result = Scanner::new(ScanConfig::for_roots(roots)).scan();

    println!("\x1b[1;36mProjectYard\x1b[0m  /  terminal view");
    println!(
        "{} projects discovered · {} folders checked\n",
        result.projects.len(),
        result.directories_visited
    );
    println!("{:<26} {:<30} PATH", "PROJECT", "TYPES");
    println!("{}", "─".repeat(88));
    for project in &result.projects {
        row(project);
    }
    println!(
        "\nInteractive filtering, keyboard navigation, and open actions are the next TUI milestone."
    );
    println!("Use `projectyard scan PATH --json` for script-friendly output.");
}

fn row(project: &Project) {
    let name = project
        .root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("/");
    let kinds = project
        .kinds
        .iter()
        .map(|kind| kind.label())
        .collect::<Vec<_>>()
        .join(", ");
    println!("{name:<26.26} {kinds:<30.30} {}", project.root.display());
}
