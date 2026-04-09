mod css;
mod html;
mod html_support;
mod model;

#[cfg(test)]
mod tests;

use crate::fixture_manifest::{FixtureSceneEntry, load_fixture_manifest};
use css::parse_css_document;
use html::parse_html_document;
pub use model::{
    CssDeclaration, CssRule, HtmlNode, HtmlNodeKind, SceneSourceGraph, SourceDocument,
    SourceDocumentKind, SourceGraphError, TraceSpan,
};
use std::fs;
use std::path::Path;

pub fn load_scene_source_graph(
    repo_root: impl AsRef<Path>,
    scene_id: &str,
) -> Result<SceneSourceGraph, SourceGraphError> {
    let manifest_path = repo_root.as_ref().join("fixtures").join("v0.1").join("manifest.toml");
    let manifest = load_fixture_manifest(&manifest_path)?;
    let scene = manifest.scene(scene_id)?;
    SceneSourceGraph::load(scene)
}

impl SceneSourceGraph {
    /// Build a [`SceneSourceGraph`] directly from HTML and CSS strings without
    /// touching the file system. Intended for testing and round-trip verification.
    pub fn from_strings(
        scene_id: &str,
        html_str: &str,
        css_str: &str,
    ) -> Result<Self, SourceGraphError> {
        use std::path::PathBuf;
        let html = SourceDocument::new_html(0, PathBuf::from("<emitted-html>"), html_str.to_owned());
        let css = SourceDocument::new_css(1, PathBuf::from("<emitted-css>"), css_str.to_owned());
        let html_nodes = parse_html_document(html.id, &html.contents)?;
        let css_rules = parse_css_document(css.id, &css.contents)?;
        Ok(Self {
            scene_id: scene_id.to_owned(),
            scene_root: PathBuf::from("."),
            html,
            css,
            html_nodes,
            css_rules,
        })
    }

    pub fn load(scene: FixtureSceneEntry) -> Result<Self, SourceGraphError> {
        let html_path = scene.manifest_root.join(&scene.scene.source_html);
        let css_path = scene.manifest_root.join(&scene.scene.source_css);
        let html_contents = fs::read_to_string(&html_path)?;
        let css_contents = fs::read_to_string(&css_path)?;
        let html = SourceDocument::new_html(0, html_path, html_contents);
        let css = SourceDocument::new_css(1, css_path, css_contents);
        let html_nodes = parse_html_document(html.id, &html.contents)?;
        let css_rules = parse_css_document(css.id, &css.contents)?;

        Ok(Self {
            scene_id: scene.scene.id,
            scene_root: scene.scene_root,
            html,
            css,
            html_nodes,
            css_rules,
        })
    }
}
