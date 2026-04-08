use crate::fixture_manifest::FixtureManifestError;
use std::path::PathBuf;

#[derive(Debug)]
pub enum SourceGraphError {
    Manifest(FixtureManifestError),
    Io(std::io::Error),
    UnsupportedHtml { reason: String, offset: usize },
    UnsupportedCss { reason: String, offset: usize },
}

impl std::fmt::Display for SourceGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manifest(error) => write!(f, "{error}"),
            Self::Io(error) => write!(f, "failed to read scene sources: {error}"),
            Self::UnsupportedHtml { reason, offset } => {
                write!(f, "unsupported HTML at byte {offset}: {reason}")
            }
            Self::UnsupportedCss { reason, offset } => {
                write!(f, "unsupported CSS at byte {offset}: {reason}")
            }
        }
    }
}

impl std::error::Error for SourceGraphError {}

impl From<FixtureManifestError> for SourceGraphError {
    fn from(error: FixtureManifestError) -> Self {
        Self::Manifest(error)
    }
}

impl From<std::io::Error> for SourceGraphError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceDocumentKind {
    Html,
    Css,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDocument {
    pub id: usize,
    pub kind: SourceDocumentKind,
    pub path: PathBuf,
    pub contents: String,
}

impl SourceDocument {
    pub fn new_html(id: usize, path: PathBuf, contents: String) -> Self {
        Self {
            id,
            kind: SourceDocumentKind::Html,
            path,
            contents,
        }
    }

    pub fn new_css(id: usize, path: PathBuf, contents: String) -> Self {
        Self {
            id,
            kind: SourceDocumentKind::Css,
            path,
            contents,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlNodeKind {
    Document,
    Element,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlNode {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub kind: HtmlNodeKind,
    pub name: Option<String>,
    pub text: Option<String>,
    pub attributes: Vec<(String, String)>,
    pub dom_path: String,
    pub document_id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssDeclaration {
    pub property: String,
    pub value: String,
    pub span: TraceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssRule {
    pub id: usize,
    pub selectors: Vec<String>,
    pub declarations: Vec<CssDeclaration>,
    pub span: TraceSpan,
    pub document_id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneSourceGraph {
    pub scene_id: String,
    pub scene_root: PathBuf,
    pub html: SourceDocument,
    pub css: SourceDocument,
    pub html_nodes: Vec<HtmlNode>,
    pub css_rules: Vec<CssRule>,
}
