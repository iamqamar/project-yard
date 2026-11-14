use std::{env, path::PathBuf, process::Command};

use projectyard_core::{ScanConfig, Scanner};
use projectyard_index::{DiscoverySettings, ProjectIndex, ScanMode, ScanSummary};
use serde::{Deserialize, Serialize};
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsInput {
    scan_locations: Vec<String>,
    excluded_directory_names: Vec<String>,
    scan_mode: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsView {
    scan_locations: Vec<String>,
    excluded_directory_names: Vec<String>,
    scan_mode: String,
}

#[tauri::command]
fn load_inventory(app: AppHandle) -> Result<Inventory, String> {
    let database = database_path(&app)?;
    let mut index = ProjectIndex::open(&database).map_err(|error| error.to_string())?;
    let settings = index
        .discovery_settings()
        .map_err(|error| error.to_string())?;
    if settings.scan_mode == ScanMode::OnLaunch && !settings.scan_locations.is_empty() {
        index
            .save_scan(&scan_roots(
                &settings.scan_locations,
                &settings.excluded_directory_names,
            ))
            .map_err(|error| error.to_string())?;
    }
    read_inventory(database)
}

#[tauri::command]
fn scan_path(app: AppHandle, path: String) -> Result<Inventory, String> {
    let root = expand_home(path.trim())?;
    if !root.is_dir() {
        return Err(format!("{} is not a readable directory", root.display()));
    }
    let database = database_path(&app)?;
    let mut index = ProjectIndex::open(&database).map_err(|error| error.to_string())?;
    let settings = index
        .discovery_settings()
        .map_err(|error| error.to_string())?;
    let result = scan_roots(&[root], &settings.excluded_directory_names);
    index
        .save_scan(&result)
        .map_err(|error| error.to_string())?;
    read_inventory(database)
}

#[tauri::command]
fn load_settings(app: AppHandle) -> Result<SettingsView, String> {
    let index = ProjectIndex::open(database_path(&app)?).map_err(|error| error.to_string())?;
    index
        .discovery_settings()
        .map(settings_view)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: SettingsInput) -> Result<SettingsView, String> {
    let settings = validate_settings(settings)?;
    let mut index = ProjectIndex::open(database_path(&app)?).map_err(|error| error.to_string())?;
    index
        .save_discovery_settings(&settings)
        .map_err(|error| error.to_string())?;
    Ok(settings_view(settings))
}

#[tauri::command]
fn scan_saved_locations(app: AppHandle) -> Result<Inventory, String> {
    let database = database_path(&app)?;
    let mut index = ProjectIndex::open(&database).map_err(|error| error.to_string())?;
    let settings = index
        .discovery_settings()
        .map_err(|error| error.to_string())?;
    if settings.scan_locations.is_empty() {
        return Err(
            "Add at least one scan location in Settings before scanning saved locations."
                .to_owned(),
        );
    }
    index
        .save_scan(&scan_roots(
            &settings.scan_locations,
            &settings.excluded_directory_names,
        ))
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

fn scan_roots(roots: &[PathBuf], exclusions: &[String]) -> projectyard_core::ScanResult {
    let mut config = ScanConfig::for_roots(roots.iter().cloned());
    config
        .excluded_directory_names
        .extend(exclusions.iter().cloned());
    Scanner::new(config).scan()
}

fn validate_settings(input: SettingsInput) -> Result<DiscoverySettings, String> {
    let scan_locations = input
        .scan_locations
        .into_iter()
        .map(|location| {
            let root = expand_home(location.trim())?;
            if !root.is_dir() {
                return Err(format!("{} is not a readable directory", root.display()));
            }
            root.canonicalize()
                .map_err(|error| format!("could not access {}: {error}", root.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let excluded_directory_names = input
        .excluded_directory_names
        .into_iter()
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .map(|name| {
            if name.contains('/') || name.contains('\\') {
                Err(format!("{name} is not a folder name"))
            } else {
                Ok(name)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let scan_mode = match input.scan_mode.as_str() {
        "on_demand" => ScanMode::OnDemand,
        "on_launch" => ScanMode::OnLaunch,
        _ => return Err("unknown scan mode".to_owned()),
    };
    Ok(DiscoverySettings {
        scan_locations,
        excluded_directory_names,
        scan_mode,
    })
}

fn settings_view(settings: DiscoverySettings) -> SettingsView {
    SettingsView {
        scan_locations: settings
            .scan_locations
            .into_iter()
            .map(|path| path.display().to_string())
            .collect(),
        excluded_directory_names: settings.excluded_directory_names,
        scan_mode: settings.scan_mode.as_str().to_owned(),
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
            load_settings,
            save_settings,
            scan_saved_locations,
            open_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running ProjectYard");
}
