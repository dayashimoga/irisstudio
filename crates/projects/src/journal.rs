use iris_core::{IrisError, IrisResult};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub sequence: u64,
    pub timestamp_utc: String,
    pub action_type: String,
    pub payload_json: String,
}

pub struct CrashJournal {
    journal_path: PathBuf,
    sequence: u64,
}

impl CrashJournal {
    pub fn new(project_path: impl AsRef<Path>) -> Self {
        let p = project_path.as_ref();
        let journal_path = p.with_extension("iris-journal");
        Self {
            journal_path,
            sequence: 0,
        }
    }

    pub fn journal_path(&self) -> &Path {
        &self.journal_path
    }

    pub fn has_unrecovered_crash(&self) -> bool {
        self.journal_path.exists()
            && fs::metadata(&self.journal_path)
                .map(|m| m.len() > 0)
                .unwrap_or(false)
    }

    pub fn append_entry(
        &mut self,
        action_type: impl Into<String>,
        payload_json: impl Into<String>,
    ) -> IrisResult<()> {
        self.sequence += 1;
        let entry = JournalEntry {
            sequence: self.sequence,
            timestamp_utc: "2026-10-19T00:00:00Z".into(),
            action_type: action_type.into(),
            payload_json: payload_json.into(),
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.journal_path)
            .map_err(|e| IrisError::IoError(format!("Failed to open crash journal: {}", e)))?;

        let line = serde_json::to_string(&entry)
            .map_err(|e| IrisError::ProjectError(e.to_string()))?
            + "\n";

        file.write_all(line.as_bytes())
            .map_err(|e| IrisError::IoError(format!("Failed to write to journal: {}", e)))?;

        file.sync_data()
            .map_err(|e| IrisError::IoError(format!("Failed to flush journal: {}", e)))?;

        Ok(())
    }

    pub fn read_entries(&self) -> IrisResult<Vec<JournalEntry>> {
        if !self.journal_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.journal_path)
            .map_err(|e| IrisError::IoError(format!("Failed to read journal: {}", e)))?;

        let mut entries = Vec::new();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<JournalEntry>(line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    pub fn clear(&mut self) -> IrisResult<()> {
        if self.journal_path.exists() {
            fs::remove_file(&self.journal_path)
                .map_err(|e| IrisError::IoError(format!("Failed to remove journal: {}", e)))?;
        }
        self.sequence = 0;
        Ok(())
    }
}
