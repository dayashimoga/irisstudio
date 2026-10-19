use iris_core::{Dimensions, IrisError, IrisResult, Rect};
use iris_effects::AdjustmentParameters;
use iris_timeline::Timeline;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectAssetKind {
    Video,
    Photo,
    Audio,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectAsset {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    pub kind: ProjectAssetKind,
    pub dimensions: Option<Dimensions>,
    pub duration_seconds: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub modified_at: String,
    pub media_assets: Vec<ProjectAsset>,
    pub timeline: Timeline,
    pub photo_adjustments: Option<AdjustmentParameters>,
    pub photo_crop: Option<Rect>,
    pub photo_straighten: f32,
}

impl Project {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            id: id.into(),
            name: name.into(),
            created_at: "2026-10-19T00:00:00Z".into(),
            modified_at: "2026-10-19T00:00:00Z".into(),
            media_assets: Vec::new(),
            timeline: Timeline::new("timeline_main", 30, 1),
            photo_adjustments: Some(AdjustmentParameters::default()),
            photo_crop: None,
            photo_straighten: 0.0,
        }
    }

    pub fn to_json(&self) -> IrisResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| IrisError::ProjectError(format!("Serialization error: {}", e)))
    }

    pub fn from_json(json: &str) -> IrisResult<Self> {
        let prj: Self = serde_json::from_str(json)
            .map_err(|e| IrisError::ProjectError(format!("Deserialization error: {}", e)))?;
        if prj.schema_version > Self::CURRENT_SCHEMA_VERSION {
            return Err(IrisError::ProjectError(format!(
                "Unsupported project version {}. Current version is {}",
                prj.schema_version,
                Self::CURRENT_SCHEMA_VERSION
            )));
        }
        Ok(prj)
    }

    /// Atomically saves the project file via a temporary write and rename.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> IrisResult<()> {
        let path = path.as_ref();
        let json = self.to_json()?;
        let temp_path = path.with_extension("tmp");

        fs::write(&temp_path, json.as_bytes())
            .map_err(|e| IrisError::IoError(format!("Failed to write temp project: {}", e)))?;

        fs::rename(&temp_path, path)
            .map_err(|e| IrisError::IoError(format!("Failed to commit project file: {}", e)))?;

        Ok(())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> IrisResult<Self> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| IrisError::IoError(format!("Failed to read project file: {}", e)))?;
        Self::from_json(&content)
    }
}
