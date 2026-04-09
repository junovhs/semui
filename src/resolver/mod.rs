//! Static-scene style resolver for the SEMUI v0.1 subset.
//!
//! Takes a [`SceneSourceGraph`] (parsed HTML + CSS) and produces a
//! [`ResolvedScene`]: every element node paired with its fully computed CSS
//! properties, after selector matching, cascade, shorthand expansion, and
//! inheritance.
//!
//! This is the output contract consumed by `EXT-01`.

mod cascade;
mod model;
mod selector;

pub use model::ComputedStyle;

use crate::{HtmlNode, HtmlNodeKind, SceneSourceGraph};
use cascade::{apply_declaration, apply_inheritance};
use selector::{selector_matches, specificity};

/// A single HTML element node paired with its fully resolved style.
#[derive(Debug, Clone)]
pub struct ResolvedNode {
    pub node: HtmlNode,
    pub style: ComputedStyle,
}

/// The full resolved scene: all element nodes in document order, each with
/// its computed style after cascade and inheritance.
///
/// Text nodes are excluded; their typography comes from their parent element.
#[derive(Debug, Clone)]
pub struct ResolvedScene {
    pub scene_id: String,
    pub nodes: Vec<ResolvedNode>,
}

/// Errors that can occur during style resolution.
#[derive(Debug)]
pub enum ResolverError {
    /// The scene has no HTML nodes at all (empty source graph).
    EmptyGraph,
}

impl std::fmt::Display for ResolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyGraph => write!(f, "scene source graph contains no HTML nodes"),
        }
    }
}

impl std::error::Error for ResolverError {}

/// Resolve all element nodes in `graph` to their computed styles.
///
/// Processing order:
/// 1. For each element node, find all matching CSS rules.
/// 2. Sort by (specificity ASC, rule index ASC) — later/higher wins on apply.
/// 3. Apply declarations in order; shorthands are expanded in place.
/// 4. Propagate inheritable properties from parent elements.
pub fn resolve_scene(graph: &SceneSourceGraph) -> Result<ResolvedScene, ResolverError> {
    if graph.html_nodes.is_empty() {
        return Err(ResolverError::EmptyGraph);
    }

    // Build a resolved-style slot for every node, keyed by node index (= node.id).
    // We process in vector order, which is document order (parent before children).
    let n = graph.html_nodes.len();
    let mut styles: Vec<Option<ComputedStyle>> = (0..n).map(|_| None).collect();

    for node in &graph.html_nodes {
        if node.kind != HtmlNodeKind::Element {
            continue;
        }

        let mut style = cascade_for_node(node, graph);
        apply_inheritance_from_parent(&mut style, node, &styles);
        styles[node.id] = Some(style);
    }

    let nodes = graph
        .html_nodes
        .iter()
        .filter(|n| n.kind == HtmlNodeKind::Element)
        .filter_map(|n| {
            styles[n.id].clone().map(|style| ResolvedNode {
                node: n.clone(),
                style,
            })
        })
        .collect();

    Ok(ResolvedScene {
        scene_id: graph.scene_id.clone(),
        nodes,
    })
}

/// Apply cascade for a single element node and return its raw computed style
/// (before inheritance).
fn cascade_for_node(node: &HtmlNode, graph: &SceneSourceGraph) -> ComputedStyle {
    // Collect matching rules. Each rule may have multiple selectors (already split
    // by the CSS parser); the winning specificity is the max across matching ones.
    let mut matches: Vec<(selector::Specificity, usize, &crate::CssRule)> = graph
        .css_rules
        .iter()
        .filter(|rule| rule.selectors.iter().any(|s| selector_matches(s, node)))
        .map(|rule| {
            let spec = rule
                .selectors
                .iter()
                .filter(|s| selector_matches(s, node))
                .map(|s| specificity(s))
                .max()
                .unwrap_or(selector::Specificity(0, 0));
            (spec, rule.id, rule)
        })
        .collect();

    // Sort ascending — declarations are applied in this order so the last one wins.
    matches.sort_by_key(|&(spec, id, _)| (spec, id));

    let mut style = ComputedStyle::default();
    for (_, _, rule) in &matches {
        for decl in &rule.declarations {
            apply_declaration(&mut style, &decl.property, &decl.value);
        }
    }
    style
}

/// Fill in inheritable fields from the parent element's already-resolved style.
fn apply_inheritance_from_parent(
    style: &mut ComputedStyle,
    node: &HtmlNode,
    styles: &[Option<ComputedStyle>],
) {
    let Some(parent_id) = node.parent_id else {
        return;
    };
    // Walk up the tree until we find a parent element that was resolved.
    // (Text nodes are skipped; document root has no parent.)
    if let Some(Some(parent_style)) = styles.get(parent_id) {
        apply_inheritance(style, parent_style);
    }
}

#[cfg(test)]
mod tests;
