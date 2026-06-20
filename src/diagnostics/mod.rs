//! Static analysis pass: report constructs that are silently dropped or
//! fall back to defaults during v0.1 normalization.
//!
//! Call [`analyze`] after building a [`SceneSourceGraph`] to receive a list
//! of [`Diagnostic`] items before (or alongside) the resolver and extractor.

#[cfg(test)]
mod tests;

use crate::CssRule;
use crate::source_graph::SceneSourceGraph;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// The coarse category of a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// A CSS property that the v0.1 resolver does not handle. The declaration
    /// is silently dropped during cascade.
    UnsupportedProperty,
    /// A CSS property is recognized but the specific value is not mapped. The
    /// property falls back to its CSS initial value.
    UnsupportedValue,
    /// A CSS selector uses syntax that v0.1 cannot match (ID, pseudo-class,
    /// pseudo-element, attribute selector, or a combinator). The entire rule is
    /// unmatched against all elements.
    UnsupportedSelector,
}

/// A single item produced by [`analyze`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub message: String,
    /// Index into `SceneSourceGraph::css_rules`.
    pub rule_id: usize,
    /// The selector that triggered this diagnostic, if applicable.
    pub selector: Option<String>,
    /// The CSS property name, if applicable.
    pub property: Option<String>,
    /// The raw CSS value, if applicable.
    pub value: Option<String>,
}

/// Analyze `graph` for unsupported constructs and return the full list of
/// diagnostics. An empty vec means the scene is within the v0.1 support
/// envelope.
pub fn analyze(graph: &SceneSourceGraph) -> Vec<Diagnostic> {
    let mut out: Vec<Diagnostic> = Vec::new();
    for rule in &graph.css_rules {
        analyze_rule(rule, &mut out);
    }
    out
}

// ---------------------------------------------------------------------------
// Per-rule analysis
// ---------------------------------------------------------------------------

fn analyze_rule(rule: &CssRule, out: &mut Vec<Diagnostic>) {
    for selector in &rule.selectors {
        if is_unsupported_selector(selector) {
            out.push(Diagnostic {
                kind: DiagnosticKind::UnsupportedSelector,
                message: format!(
                    "selector '{selector}' uses syntax not supported in v0.1 \
                     (id/pseudo/attribute/combinator); rule is unmatched"
                ),
                rule_id: rule.id,
                selector: Some(selector.clone()),
                property: None,
                value: None,
            });
        }
    }

    for decl in &rule.declarations {
        if !is_known_property(&decl.property) {
            out.push(Diagnostic {
                kind: DiagnosticKind::UnsupportedProperty,
                message: format!(
                    "property '{}' is not supported in v0.1 and will be dropped",
                    decl.property
                ),
                rule_id: rule.id,
                selector: rule.selectors.first().cloned(),
                property: Some(decl.property.clone()),
                value: Some(decl.value.clone()),
            });
        } else if let Some(msg) = unsupported_value_message(&decl.property, &decl.value) {
            out.push(Diagnostic {
                kind: DiagnosticKind::UnsupportedValue,
                message: msg,
                rule_id: rule.id,
                selector: rule.selectors.first().cloned(),
                property: Some(decl.property.clone()),
                value: Some(decl.value.clone()),
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Selector analysis
// ---------------------------------------------------------------------------

/// Returns `true` when the selector uses patterns v0.1 cannot match.
fn is_unsupported_selector(selector: &str) -> bool {
    let s = selector.trim();
    // Internal whitespace = descendant/child combinator
    if s.chars().any(char::is_whitespace) {
        return true;
    }
    // Explicit combinators or advanced syntax
    s.contains(['#', ':', '[', ']', '>', '+', '~'])
}

// ---------------------------------------------------------------------------
// Property / value analysis
// ---------------------------------------------------------------------------

const KNOWN_PROPERTIES: &[&str] = &[
    "margin",
    "padding",
    "background",
    "border",
    "position",
    "display",
    "box-sizing",
    "top",
    "left",
    "width",
    "height",
    "min-width",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "border-width",
    "border-color",
    "border-radius",
    "background-color",
    "flex-direction",
    "align-items",
    "justify-content",
    "align-self",
    "gap",
    "cursor",
    "color",
    "font-family",
    "font-size",
    "font-weight",
    "line-height",
];

fn is_known_property(property: &str) -> bool {
    KNOWN_PROPERTIES.contains(&property)
}

/// Returns a diagnostic message when a known property has a value we cannot
/// map. Currently checks `position` and `display` — the highest-impact
/// properties to silently mis-render.
fn unsupported_value_message(property: &str, value: &str) -> Option<String> {
    match property {
        "position" => match value {
            "static" | "absolute" | "inherit" => None,
            _ => Some(format!(
                "position: '{value}' is not supported in v0.1; treated as static"
            )),
        },
        "display" => match value {
            "block" | "flex" | "inline-flex" | "none" | "inherit" => None,
            _ => Some(format!(
                "display: '{value}' is not supported in v0.1; treated as block"
            )),
        },
        _ => None,
    }
}
