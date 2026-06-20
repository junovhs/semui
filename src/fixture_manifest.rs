mod parse;

use parse::ParsedManifest;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum FixtureManifestError {
    Io(std::io::Error),
    Parse { line: usize, message: String },
    SceneNotFound { scene_id: String },
}

impl std::fmt::Display for FixtureManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "failed to read fixture manifest: {error}"),
            Self::Parse { line, message } => {
                write!(
                    f,
                    "failed to parse fixture manifest at line {line}: {message}"
                )
            }
            Self::SceneNotFound { scene_id } => {
                write!(
                    f,
                    "fixture scene `{scene_id}` was not found in the manifest"
                )
            }
        }
    }
}

impl std::error::Error for FixtureManifestError {}

impl From<std::io::Error> for FixtureManifestError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureManifest {
    manifest_path: PathBuf,
    pub schema_version: u32,
    pub corpus: String,
    pub scenes: Vec<FixtureScene>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureScene {
    pub id: String,
    pub priority: u32,
    /// Scene directory relative to the manifest root. All scene artifacts live
    /// at conventional paths under this directory (see [`FixtureScene::source_html`]).
    pub dir: PathBuf,
    pub tags: Vec<String>,
}

impl FixtureScene {
    /// Source HTML path, relative to `dir`: `<dir>/source.html`.
    pub fn source_html(&self) -> PathBuf {
        self.dir.join("source.html")
    }

    /// Source CSS path, relative to `dir`: `<dir>/source.css`.
    pub fn source_css(&self) -> PathBuf {
        self.dir.join("source.css")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureSceneEntry {
    pub manifest_root: PathBuf,
    pub scene_root: PathBuf,
    pub scene: FixtureScene,
}

pub fn load_fixture_manifest(
    path: impl AsRef<Path>,
) -> Result<FixtureManifest, FixtureManifestError> {
    FixtureManifest::load(path)
}

impl FixtureManifest {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, FixtureManifestError> {
        let manifest_path = path.as_ref().to_path_buf();
        let contents = fs::read_to_string(&manifest_path)?;
        let parsed = ParsedManifest::parse(&contents)?;

        Ok(Self {
            manifest_path,
            schema_version: parsed.schema_version,
            corpus: parsed.corpus,
            scenes: parsed.scenes,
        })
    }

    pub fn manifest_root(&self) -> &Path {
        self.manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
    }

    pub fn scene(&self, scene_id: &str) -> Result<FixtureSceneEntry, FixtureManifestError> {
        let scene = self
            .scenes
            .iter()
            .find(|scene| scene.id == scene_id)
            .cloned()
            .ok_or_else(|| FixtureManifestError::SceneNotFound {
                scene_id: scene_id.to_owned(),
            })?;
        let manifest_root = self.manifest_root().to_path_buf();
        let scene_root = manifest_root.join(&scene.dir);

        Ok(FixtureSceneEntry {
            manifest_root,
            scene_root,
            scene,
        })
    }
}
