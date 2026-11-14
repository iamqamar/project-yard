use std::{env, path::PathBuf, process::Command};

use projectyard_core::{ScanConfig, Scanner};
use projectyard_index::{ProjectIndex, ScanSummary};
use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectRecord {
    root: String,
    kinds: Vec<String>,
    markers: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Inventory {
    database: String,
    projects: Vec<ProjectRecord>,
    last_scan_unix_seconds: Option<i64>,
    directories_visited: Option<usize>,
    issue_count: Option<usize>,
}

#[tauri::command]
fn load_inventory(app: AppHandle) -> Result<Inventory, String> {
    read_inventory(database_path(&app)?)
}

#[tauri::command]
fn scan_path(app: AppHandle, path: String) -> Result<Inventory, String> {
    let root = expand_home(path.trim())?;
    if !root.is_dir() {
        return Err(format!("{} is not a readable directory", root.display()));
    }
    let result = Scanner::new(ScanConfig::for_roots([root])).scan();
    let database = database_path(&app)?;
    let mut index = ProjectIndex::open(&database).map_err(|error| error.to_string())?;
    index
        .save_scan(&result)
        .map_err(|error| error.to_string())?;
    read_inventory(database)
}

#[tauri::command]
fn open_project(path: String, target: String) -> Result<String, String> {
    let root = expand_home(path.trim())?
        .canonicalize()
        .map_err(|error| format!("could not access {path}: {error}"))?;
    if !root.is_dir() {
        return Err(format!("{} is not a directory", root.display()));
    }

    match target.as_str() {
        "folder" => {
            Command::new("xdg-open")
                .arg(&root)
                .spawn()
                .map_err(|error| format!("could not open the folder: {error}"))?;
            Ok(format!("Opened {} in your file manager.", root.display()))
        }
        "zed" => {
            Command::new("zed")
                .arg(&root)
                .spawn()
                .map_err(|error| format!("could not start Zed: {error}. Install Zed's command-line launcher, then try again."))?;
            Ok(format!("Opened {} in Zed.", root.display()))
        }
        _ => Err("unknown project action".to_owned()),
    }
}

fn read_inventory(database: PathBuf) -> Result<Inventory, String> {
    let index = ProjectIndex::open(&database).map_err(|error| error.to_string())?;
    let summary = index.latest_scan().map_err(|error| error.to_string())?;
    let projects = index
        .projects()
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|project| ProjectRecord {
            root: project.root.display().to_string(),
            kinds: project
                .kinds
                .into_iter()
                .map(|kind| kind.label().to_owned())
                .collect(),
            markers: project.markers,
        })
        .collect();
    Ok(inventory(database, projects, summary))
}

fn inventory(
    database: PathBuf,
    projects: Vec<ProjectRecord>,
    summary: Option<ScanSummary>,
) -> Inventory {
    Inventory {
        database: database.display().to_string(),
        projects,
        last_scan_unix_seconds: summary.map(|scan| scan.completed_unix_seconds),
        directories_visited: summary.map(|scan| scan.directories_visited),
        issue_count: summary.map(|scan| scan.issue_count),
    }
}

fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    Ok(directory.join("projectyard.sqlite"))
}

fn expand_home(value: &str) -> Result<PathBuf, String> {
    if value == "~" || value.starts_with("~/") {
        let home = env::var_os("HOME").ok_or("could not determine the home directory")?;
        return Ok(PathBuf::from(home).join(value.trim_start_matches("~/")));
    }
    Ok(PathBuf::from(value))
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_inventory,
            scan_path,
            open_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running ProjectYard");
}
