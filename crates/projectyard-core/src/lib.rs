//! Filesystem discovery for ProjectYard.
//!
//! This crate intentionally has no UI or database dependency. Both frontends
//! can use the same transparent detector output.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

/// A technology or local tool found in a project directory.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProjectKind {
    Android,
    Cargo,
    Cursor,
    Fleet,
    Flutter,
    Git,
    Go,
    Gradle,
    JetBrains,
    Kotlin,
    Maven,
    Node,
    Php,
    Python,
    Ruby,
    Uv,
    VsCode,
    Xcode,
    Zed,
}

impl ProjectKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Android => "Android",
            Self::Cargo => "Rust",
            Self::Cursor => "Cursor",
            Self::Fleet => "Fleet",
            Self::Flutter => "Flutter",
            Self::Git => "Git",
            Self::Go => "Go",
            Self::Gradle => "Gradle",
            Self::JetBrains => "JetBrains",
            Self::Kotlin => "Kotlin",
            Self::Maven => "Maven",
            Self::Node => "Node.js",
            Self::Php => "PHP",
            Self::Python => "Python",
            Self::Ruby => "Ruby",
            Self::Uv => "uv",
            Self::VsCode => "VS Code",
            Self::Xcode => "Xcode",
            Self::Zed => "Zed",
        }
    }

    #[must_use]
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "Android" => Some(Self::Android),
            "Rust" => Some(Self::Cargo),
            "Cursor" => Some(Self::Cursor),
            "Fleet" => Some(Self::Fleet),
            "Flutter" => Some(Self::Flutter),
            "Git" => Some(Self::Git),
            "Go" => Some(Self::Go),
            "Gradle" => Some(Self::Gradle),
            "JetBrains" => Some(Self::JetBrains),
            "Kotlin" => Some(Self::Kotlin),
            "Maven" => Some(Self::Maven),
            "Node.js" => Some(Self::Node),
            "PHP" => Some(Self::Php),
            "Python" => Some(Self::Python),
            "Ruby" => Some(Self::Ruby),
            "uv" => Some(Self::Uv),
            "VS Code" => Some(Self::VsCode),
            "Xcode" => Some(Self::Xcode),
            "Zed" => Some(Self::Zed),
            _ => None,
        }
    }
}

/// Concrete evidence supporting a project tag.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evidence {
    pub kind: ProjectKind,
    pub marker: String,
}

/// A directory that contains one or more recognized project markers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Project {
    pub root: PathBuf,
    pub kinds: Vec<ProjectKind>,
    pub evidence: Vec<Evidence>,
    pub modified: Option<SystemTime>,
}

/// A readable problem from a scan that does not stop other folders scanning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanIssue {
    pub path: PathBuf,
    pub message: String,
}

/// Tunable traversal rules. All scanning is local and does not follow links.
#[derive(Clone, Debug)]
pub struct ScanConfig {
    pub roots: Vec<PathBuf>,
    pub excluded_directory_names: BTreeSet<String>,
    pub max_depth: usize,
}

impl ScanConfig {
    #[must_use]
    pub fn for_roots(roots: impl IntoIterator<Item = PathBuf>) -> Self {
        Self {
            roots: roots.into_iter().collect(),
            excluded_directory_names: default_exclusions(),
            max_depth: 24,
        }
    }
}

/// The complete output of a scan.
#[derive(Clone, Debug, Default)]
pub struct ScanResult {
    pub projects: Vec<Project>,
    pub issues: Vec<ScanIssue>,
    pub directories_visited: usize,
}

/// Discovers projects without uploading or persisting any filesystem data.
#[derive(Clone, Debug)]
pub struct Scanner {
    config: ScanConfig,
}

impl Scanner {
    #[must_use]
    pub const fn new(config: ScanConfig) -> Self {
        Self { config }
    }

    #[must_use]
    pub fn config(&self) -> &ScanConfig {
        &self.config
    }

    #[must_use]
    pub fn scan(&self) -> ScanResult {
        let mut result = ScanResult::default();
        for root in &self.config.roots {
            self.walk(root, 0, &mut result);
        }
        result
            .projects
            .sort_by(|left, right| left.root.cmp(&right.root));
        result
    }

    fn walk(&self, path: &Path, depth: usize, result: &mut ScanResult) {
        if depth > self.config.max_depth {
            return;
        }
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) => {
                result.issues.push(ScanIssue {
                    path: path.to_path_buf(),
                    message: error.to_string(),
                });
                return;
            }
        };
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return;
        }

        result.directories_visited += 1;
        if let Some(project) = detect_project(path) {
            result.projects.push(project);
        }

        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(error) => {
                result.issues.push(ScanIssue {
                    path: path.to_path_buf(),
                    message: error.to_string(),
                });
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    result.issues.push(ScanIssue {
                        path: path.to_path_buf(),
                        message: error.to_string(),
                    });
                    continue;
                }
            };
            let child = entry.path();
            if self.is_excluded_directory(&child) {
                continue;
            }
            self.walk(&child, depth + 1, result);
        }
    }

    fn is_excluded_directory(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| self.config.excluded_directory_names.contains(name))
    }
}

/// Detect a project at one exact directory without walking children.
#[must_use]
pub fn detect_project(root: &Path) -> Option<Project> {
    let mut evidence = Vec::new();
    let has = |marker: &str| root.join(marker).exists();
    let file_has = |marker: &str, needle: &str| {
        fs::read_to_string(root.join(marker)).is_ok_and(|contents| contents.contains(needle))
    };
    let mut add = |kind, marker: &str| {
        if !evidence.iter().any(|item: &Evidence| item.kind == kind) {
            evidence.push(Evidence {
                kind,
                marker: marker.to_owned(),
            });
        }
    };

    if has("Cargo.toml") {
        add(ProjectKind::Cargo, "Cargo.toml");
    }
    if has("package.json") {
        add(ProjectKind::Node, "package.json");
    }
    if has("pyproject.toml") || has("requirements.txt") || has("setup.py") {
        add(
            ProjectKind::Python,
            if has("pyproject.toml") {
                "pyproject.toml"
            } else if has("requirements.txt") {
                "requirements.txt"
            } else {
                "setup.py"
            },
        );
    }
    if has("uv.lock") {
        add(ProjectKind::Uv, "uv.lock");
    }
    if has("pubspec.yaml") && file_has("pubspec.yaml", "flutter") {
        add(ProjectKind::Flutter, "pubspec.yaml");
    }
    if has("settings.gradle")
        || has("settings.gradle.kts")
        || has("build.gradle")
        || has("build.gradle.kts")
    {
        let marker = if has("settings.gradle.kts") {
            "settings.gradle.kts"
        } else if has("settings.gradle") {
            "settings.gradle"
        } else if has("build.gradle.kts") {
            "build.gradle.kts"
        } else {
            "build.gradle"
        };
        add(ProjectKind::Gradle, marker);
        if marker.ends_with(".kts") {
            add(ProjectKind::Kotlin, marker);
        }
    }
    if has("AndroidManifest.xml") || root.join("app/src/main/AndroidManifest.xml").exists() {
        add(ProjectKind::Android, "AndroidManifest.xml");
    }
    if has("pom.xml") {
        add(ProjectKind::Maven, "pom.xml");
    }
    if has("go.mod") {
        add(ProjectKind::Go, "go.mod");
    }
    if has("Gemfile") {
        add(ProjectKind::Ruby, "Gemfile");
    }
    if has("composer.json") {
        add(ProjectKind::Php, "composer.json");
    }
    if has(".xcodeproj") || directory_has_suffix(root, ".xcodeproj") {
        add(ProjectKind::Xcode, ".xcodeproj");
    }
    if has(".git") {
        add(ProjectKind::Git, ".git");
    }
    if has(".zed") {
        add(ProjectKind::Zed, ".zed");
    }
    if has(".vscode") {
        add(ProjectKind::VsCode, ".vscode");
    }
    if has(".idea") {
        add(ProjectKind::JetBrains, ".idea");
    }
    if has(".fleet") {
        add(ProjectKind::Fleet, ".fleet");
    }
    if has(".cursor") {
        add(ProjectKind::Cursor, ".cursor");
    }

    if evidence.is_empty() {
        return None;
    }
    let mut kinds: Vec<_> = evidence.iter().map(|item| item.kind).collect();
    kinds.sort_unstable();
    Some(Project {
        root: root.to_path_buf(),
        kinds,
        evidence,
        modified: fs::metadata(root)
            .and_then(|metadata| metadata.modified())
            .ok(),
    })
}

fn directory_has_suffix(root: &Path, suffix: &str) -> bool {
    fs::read_dir(root).is_ok_and(|entries| {
        entries.flatten().any(|entry| {
            entry.file_type().is_ok_and(|type_| type_.is_dir())
                && entry.file_name().to_string_lossy().ends_with(suffix)
        })
    })
}

fn default_exclusions() -> BTreeSet<String> {
    [
        ".git",
        ".gradle",
        ".dart_tool",
        ".venv",
        "__pycache__",
        "build",
        "dist",
        "node_modules",
        "Pods",
        "target",
        "venv",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_multiple_markers_without_collapsing_them() {
        let root = std::env::temp_dir().join(format!("projectyard-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".zed")).expect("create zed evidence");
        fs::write(root.join("Cargo.toml"), "[package]").expect("write cargo marker");
        fs::write(root.join("uv.lock"), "version = 1").expect("write uv marker");

        let project = detect_project(&root).expect("detected project");
        assert!(project.kinds.contains(&ProjectKind::Cargo));
        assert!(project.kinds.contains(&ProjectKind::Uv));
        assert!(project.kinds.contains(&ProjectKind::Zed));
        assert_eq!(project.evidence.len(), 3);
        fs::remove_dir_all(root).expect("clean fixture");
    }

    #[test]
    fn defaults_skip_dependency_directories() {
        let config = ScanConfig::for_roots([PathBuf::from("/tmp")]);
        assert!(config.excluded_directory_names.contains("node_modules"));
        assert!(config.excluded_directory_names.contains("target"));
    }
}
