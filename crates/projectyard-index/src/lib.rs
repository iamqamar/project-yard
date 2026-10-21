//! Private, local SQLite storage for ProjectYard scan results.

use std::{
    fmt, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use projectyard_core::{Project, ProjectKind, ScanResult};
use rusqlite::{Connection, params};

/// A persisted project representation tailored to list and map views.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedProject {
    pub root: PathBuf,
    pub kinds: Vec<ProjectKind>,
    pub markers: Vec<String>,
    pub modified_unix_seconds: Option<i64>,
    pub indexed_unix_seconds: i64,
}

/// Summary for the most recent completed local scan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScanSummary {
    pub completed_unix_seconds: i64,
    pub project_count: usize,
    pub directories_visited: usize,
    pub issue_count: usize,
}

/// An error emitted by ProjectYard's local data store.
#[derive(Debug)]
pub enum IndexError {
    Io(std::io::Error),
    Sql(rusqlite::Error),
    UnknownProjectKind(String),
}

impl fmt::Display for IndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "filesystem error: {error}"),
            Self::Sql(error) => write!(formatter, "database error: {error}"),
            Self::UnknownProjectKind(kind) => {
                write!(formatter, "unknown stored project kind: {kind}")
            }
        }
    }
}

impl std::error::Error for IndexError {}

impl From<std::io::Error> for IndexError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for IndexError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sql(error)
    }
}

/// SQLite index kept wholly on the user's local machine.
#[derive(Debug)]
pub struct ProjectIndex {
    connection: Connection,
}

impl ProjectIndex {
    /// Opens or creates an index at `database_path` and applies its schema.
    pub fn open(database_path: impl AsRef<Path>) -> Result<Self, IndexError> {
        let database_path = database_path.as_ref();
        if let Some(parent) = database_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(database_path)?;
        connection.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS scan_runs (
                id INTEGER PRIMARY KEY,
                completed_unix_seconds INTEGER NOT NULL,
                project_count INTEGER NOT NULL,
                directories_visited INTEGER NOT NULL,
                issue_count INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS projects (
                root TEXT PRIMARY KEY NOT NULL,
                kinds TEXT NOT NULL,
                markers TEXT NOT NULL,
                modified_unix_seconds INTEGER,
                indexed_unix_seconds INTEGER NOT NULL
            );
            ",
        )?;
        Ok(Self { connection })
    }

    /// Replaces the current inventory atomically and records a completed scan.
    pub fn save_scan(&mut self, result: &ScanResult) -> Result<ScanSummary, IndexError> {
        let completed_unix_seconds = unix_seconds(SystemTime::now());
        let summary = ScanSummary {
            completed_unix_seconds,
            project_count: result.projects.len(),
            directories_visited: result.directories_visited,
            issue_count: result.issues.len(),
        };
        let transaction = self.connection.transaction()?;
        transaction.execute("DELETE FROM projects", [])?;
        {
            let mut statement = transaction.prepare(
                "INSERT INTO projects (
                    root, kinds, markers, modified_unix_seconds, indexed_unix_seconds
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            for project in &result.projects {
                statement.execute(params![
                    project.root.to_string_lossy(),
                    encode_kinds(project),
                    encode_markers(project),
                    project.modified.map(unix_seconds),
                    completed_unix_seconds,
                ])?;
            }
        }
        transaction.execute(
            "INSERT INTO scan_runs (
                completed_unix_seconds, project_count, directories_visited, issue_count
             ) VALUES (?1, ?2, ?3, ?4)",
            params![
                summary.completed_unix_seconds,
                saturating_sql_count(summary.project_count),
                saturating_sql_count(summary.directories_visited),
                saturating_sql_count(summary.issue_count),
            ],
        )?;
        transaction.commit()?;
        Ok(summary)
    }

    /// Lists the current inventory by path.
    pub fn projects(&self) -> Result<Vec<IndexedProject>, IndexError> {
        let mut statement = self.connection.prepare(
            "SELECT root, kinds, markers, modified_unix_seconds, indexed_unix_seconds
             FROM projects ORDER BY root COLLATE NOCASE",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<i64>>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?;
        let mut projects = Vec::new();
        for row in rows {
            let (root, kinds, markers, modified_unix_seconds, indexed_unix_seconds) = row?;
            projects.push(IndexedProject {
                root: PathBuf::from(root),
                kinds: decode_kinds(&kinds)?,
                markers: decode_values(&markers),
                modified_unix_seconds,
                indexed_unix_seconds,
            });
        }
        Ok(projects)
    }

    /// Returns the latest scan summary, if this index has ever been scanned.
    pub fn latest_scan(&self) -> Result<Option<ScanSummary>, IndexError> {
        self.connection
            .query_row(
                "SELECT completed_unix_seconds, project_count, directories_visited, issue_count
                 FROM scan_runs ORDER BY id DESC LIMIT 1",
                [],
                |row| {
                    Ok(ScanSummary {
                        completed_unix_seconds: row.get(0)?,
                        project_count: row.get::<_, i64>(1)?.max(0) as usize,
                        directories_visited: row.get::<_, i64>(2)?.max(0) as usize,
                        issue_count: row.get::<_, i64>(3)?.max(0) as usize,
                    })
                },
            )
            .map(Some)
            .or_else(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(IndexError::Sql(other)),
            })
    }
}

fn encode_kinds(project: &Project) -> String {
    project
        .kinds
        .iter()
        .map(|kind| kind.label())
        .collect::<Vec<_>>()
        .join("\u{1f}")
}

fn encode_markers(project: &Project) -> String {
    project
        .evidence
        .iter()
        .map(|evidence| evidence.marker.as_str())
        .collect::<Vec<_>>()
        .join("\u{1f}")
}

fn decode_kinds(value: &str) -> Result<Vec<ProjectKind>, IndexError> {
    decode_values(value)
        .into_iter()
        .map(|kind| ProjectKind::from_label(&kind).ok_or(IndexError::UnknownProjectKind(kind)))
        .collect()
}

fn decode_values(value: &str) -> Vec<String> {
    value
        .split('\u{1f}')
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect()
}

fn unix_seconds(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .try_into()
        .unwrap_or(i64::MAX)
}

fn saturating_sql_count(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use projectyard_core::{Evidence, ProjectKind};

    #[test]
    fn saves_and_loads_a_local_inventory() {
        let path =
            std::env::temp_dir().join(format!("projectyard-index-{}.sqlite", std::process::id()));
        let _ = fs::remove_file(&path);
        let mut index = ProjectIndex::open(&path).expect("open index");
        let result = ScanResult {
            projects: vec![Project {
                root: PathBuf::from("/work/projectyard"),
                kinds: vec![ProjectKind::Cargo, ProjectKind::Zed],
                evidence: vec![
                    Evidence {
                        kind: ProjectKind::Cargo,
                        marker: "Cargo.toml".to_owned(),
                    },
                    Evidence {
                        kind: ProjectKind::Zed,
                        marker: ".zed".to_owned(),
                    },
                ],
                modified: None,
            }],
            issues: Vec::new(),
            directories_visited: 5,
        };

        let summary = index.save_scan(&result).expect("save scan");
        assert_eq!(summary.project_count, 1);
        assert_eq!(index.latest_scan().expect("latest scan"), Some(summary));
        let projects = index.projects().expect("projects");
        assert_eq!(projects.len(), 1);
        assert_eq!(
            projects[0].kinds,
            vec![ProjectKind::Cargo, ProjectKind::Zed]
        );
        drop(index);
        let _ = fs::remove_file(path);
    }
}
