use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::config::AppConfig;

pub struct TaskRecord {
    pub task_id: String,
    pub prompt: String,
    pub model: String,
    pub status: String,
    pub video_url: Option<String>,
    pub output_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct TaskStore {
    conn: Connection,
}

impl TaskStore {
    /// Open an in-memory database (for testing)
    #[allow(dead_code)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_tables()?;
        Ok(store)
    }

    pub fn open() -> Result<Self> {
        let path = Self::db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)
            .with_context(|| format!("failed to open database at {}", path.display()))?;
        let store = Self { conn };
        store.init_tables()?;
        Ok(store)
    }

    fn db_path() -> Result<PathBuf> {
        Ok(AppConfig::config_dir()?.join("tasks.db"))
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks (
                task_id    TEXT PRIMARY KEY,
                prompt     TEXT NOT NULL,
                model      TEXT NOT NULL DEFAULT '',
                status     TEXT NOT NULL DEFAULT 'submitted',
                video_url  TEXT,
                output_path TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;
        Ok(())
    }

    pub fn insert(&self, task_id: &str, prompt: &str, model: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tasks (task_id, prompt, model, status) VALUES (?1, ?2, ?3, 'submitted')",
            params![task_id, prompt, model],
        )?;
        Ok(())
    }

    pub fn update_status(&self, task_id: &str, status: &str, video_url: Option<&str>) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET status = ?1, video_url = ?2, updated_at = datetime('now') WHERE task_id = ?3",
            params![status, video_url, task_id],
        )?;
        Ok(())
    }

    pub fn update_output_path(&self, task_id: &str, path: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET output_path = ?1, updated_at = datetime('now') WHERE task_id = ?2",
            params![path, task_id],
        )?;
        Ok(())
    }

    pub fn list(&self, limit: usize, status_filter: Option<&str>) -> Result<Vec<TaskRecord>> {
        let mut results = Vec::new();

        if let Some(s) = status_filter.filter(|s| *s != "all") {
            let mut stmt = self.conn.prepare(
                "SELECT task_id, prompt, model, status, video_url, output_path, created_at, updated_at \
                 FROM tasks WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![s, limit], Self::map_row)?;
            for row in rows {
                results.push(row?);
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT task_id, prompt, model, status, video_url, output_path, created_at, updated_at \
                 FROM tasks ORDER BY created_at DESC LIMIT ?1",
            )?;
            let rows = stmt.query_map(params![limit], Self::map_row)?;
            for row in rows {
                results.push(row?);
            }
        }

        Ok(results)
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<TaskRecord> {
        Ok(TaskRecord {
            task_id: row.get(0)?,
            prompt: row.get(1)?,
            model: row.get(2)?,
            status: row.get(3)?,
            video_url: row.get(4)?,
            output_path: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }
}
