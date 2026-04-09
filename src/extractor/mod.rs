//! Extracts a [`SceneIr`] from a [`LaidOutScene`] + [`SceneSourceGraph`].
//!
//! The entry point is [`extract_ir`], which walks the HTML node list in
//! document order and maps each visible element or text run to an [`IrNode`].

mod map;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use crate::{HtmlNode, HtmlNodeKind};
use crate::ir::{
    ControlKind, ExecutionMode, IrNode, NodeKind, SceneIr, SourceRef,
};
use crate::layout::{LaidOutNode, LaidOutScene};
use crate::source_graph::SceneSourceGraph;

/// Tags that are invisible structural boilerplate — skipped entirely along
/// with all of their descendants.
const INVISIBLE_TAGS: &[&str] = &["head", "meta", "link", "title", "style", "script"];

/// Tags that are transparent wrappers: we traverse their children but do not
/// emit an [`IrNode`] for the element itself. A transparent element's children
/// inherit their parent IR id from the transparent element's own parent.
/// This ensures `<html>` and `<body>` vanish from the IR so that re-parsing
/// the emitted HTML (which always has those wrappers) produces the same IR.
const TRANSPARENT_TAGS: &[&str] = &["html", "body"];

#[derive(Debug)]
pub enum ExtractorError {
    /// The laid-out scene and the source graph disagree on their scene ids.
    SceneIdMismatch { laid_out: String, graph: String },
}

impl std::fmt::Display for ExtractorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SceneIdMismatch { laid_out, graph } => write!(
                f,
                "scene id mismatch: laid_out={laid_out:?} graph={graph:?}"
            ),
        }
    }
}

impl std::error::Error for ExtractorError {}

/// Extract a [`SceneIr`] from the combination of a fully laid-out scene and its
/// source graph.
///
/// Nodes are emitted in pre-order (parent before children), mirroring document
/// order in the source HTML.
pub fn extract_ir(
    laid_out: &LaidOutScene,
    graph: &SceneSourceGraph,
) -> Result<SceneIr, ExtractorError> {
    if laid_out.scene_id != graph.scene_id {
        return Err(ExtractorError::SceneIdMismatch {
            laid_out: laid_out.scene_id.clone(),
            graph: graph.scene_id.clone(),
        });
    }

    let node_map: HashMap<usize, &LaidOutNode> =
        laid_out.nodes.iter().map(|n| (n.node.id, n)).collect();

    let mut skipped_ids: HashSet<usize> = HashSet::new();
    let mut html_to_ir_id: HashMap<usize, String> = HashMap::new();
    let mut ir_nodes: Vec<IrNode> = Vec::new();

    for html_node in &graph.html_nodes {
        if let Some(pid) = html_node.parent_id
            && skipped_ids.contains(&pid)
        {
            skipped_ids.insert(html_node.id);
            continue;
        }

        match html_node.kind {
            HtmlNodeKind::Document => continue,
            HtmlNodeKind::Element => {
                process_element(
                    html_node,
                    &node_map,
                    &mut html_to_ir_id,
                    &mut ir_nodes,
                    &mut skipped_ids,
                );
            }
            HtmlNodeKind::Text => {
                if let Some(node) = process_text(
                    html_node,
                    &node_map,
                    &html_to_ir_id,
                    ir_nodes.len(),
                    &skipped_ids,
                ) {
                    ir_nodes.push(node);
                }
            }
        }
    }

    Ok(SceneIr {
        schema_version: 1,
        scene_id: laid_out.scene_id.clone(),
        corpus: "v0.1".to_owned(),
        execution_mode: ExecutionMode::Static,
        nodes: ir_nodes,
    })
}

fn process_element(
    html_node: &HtmlNode,
    node_map: &HashMap<usize, &LaidOutNode>,
    html_to_ir_id: &mut HashMap<usize, String>,
    ir_nodes: &mut Vec<IrNode>,
    skipped_ids: &mut HashSet<usize>,
) {
    let tag = html_node.name.as_deref().unwrap_or("");
    if INVISIBLE_TAGS.contains(&tag) {
        skipped_ids.insert(html_node.id);
        return;
    }

    if TRANSPARENT_TAGS.contains(&tag) {
        if let Some(pir) = html_node
            .parent_id
            .and_then(|pid| html_to_ir_id.get(&pid))
            .cloned()
        {
            html_to_ir_id.insert(html_node.id, pir);
        }
        return;
    }

    let Some(lo) = node_map.get(&html_node.id) else { return; };

    let ir_id = format!("n{}", ir_nodes.len());
    html_to_ir_id.insert(html_node.id, ir_id.clone());

    let parent_ir_id = html_node
        .parent_id
        .and_then(|pid| html_to_ir_id.get(&pid))
        .cloned();

    let (kind, control_kind) = element_kind(tag);

    ir_nodes.push(IrNode {
        id: ir_id,
        kind,
        parent_id: parent_ir_id,
        control_kind,
        text_content: None,
        layout: map::to_layout(&lo.style, &lo.geometry),
        paint: map::to_paint(&lo.style),
        typography: map::to_typography(&lo.style),
        source: SourceRef {
            doc_id: html_node.document_id,
            dom_path: html_node.dom_path.clone(),
            span: None,
        },
    });
}

fn process_text(
    html_node: &HtmlNode,
    node_map: &HashMap<usize, &LaidOutNode>,
    html_to_ir_id: &HashMap<usize, String>,
    next_id: usize,
    skipped_ids: &HashSet<usize>,
) -> Option<IrNode> {
    let text = html_node.text.as_ref()?.trim().to_owned();
    if text.is_empty() {
        return None;
    }

    let parent_id = match html_node.parent_id {
        Some(pid) if !skipped_ids.contains(&pid) => pid,
        _ => return None,
    };

    let parent_ir_id = html_to_ir_id.get(&parent_id)?.clone();

    let typography = node_map
        .get(&parent_id)
        .and_then(|lo| map::to_typography(&lo.style));

    Some(IrNode {
        id: format!("n{next_id}"),
        kind: NodeKind::Text,
        parent_id: Some(parent_ir_id),
        control_kind: None,
        text_content: Some(text),
        layout: map::to_layout_default(),
        paint: map::to_paint_default(),
        typography,
        source: SourceRef {
            doc_id: html_node.document_id,
            dom_path: html_node.dom_path.clone(),
            span: None,
        },
    })
}

fn element_kind(tag: &str) -> (NodeKind, Option<ControlKind>) {
    match tag {
        "button" => (NodeKind::Control, Some(ControlKind::Button)),
        _ => (NodeKind::Box, None),
    }
}
