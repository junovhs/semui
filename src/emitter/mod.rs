//! Emit minimal HTML and CSS from a [`SceneIr`] (strict mode, v0.1).
//!
//! Entry point: [`emit`]. The result is an [`EmittedScene`] containing
//! separate HTML and CSS strings ready to write to disk or compare in tests.

pub(crate) mod css;
pub(crate) mod html;

#[cfg(test)]
mod tests;

use crate::ir::SceneIr;
use crate::target::{TargetCapabilities, TargetEmission, TargetEmitter, capability_gaps};

/// The output of the strict emitter: a full HTML document and a CSS stylesheet.
#[derive(Debug, Clone, PartialEq)]
pub struct EmittedScene {
    /// A complete `<!doctype html>` document. CSS is referenced via
    /// `<link rel="stylesheet" href="styles.css">`.
    pub html: String,
    /// A flat CSS stylesheet. All rules use generated class selectors
    /// (e.g. `.n0`, `.n1`) matching the `id` fields in the source [`SceneIr`].
    pub css: String,
}

/// Emit a minimal HTML document and CSS stylesheet from `ir`.
///
/// # Guarantees
///
/// - Every Box and Control node in `ir.nodes` appears as an HTML element
///   with `class="{node.id}"` and a corresponding CSS rule.
/// - Text nodes appear as raw text content inside their parent element.
/// - Only properties that are set (non-default) in the IR are emitted.
pub fn emit(ir: &SceneIr) -> EmittedScene {
    EmittedScene {
        html: html::build_html(ir),
        css: css::build_css(&ir.nodes),
    }
}

/// The HTML reference target: the [`TargetEmitter`] adapter over the strict
/// emitter.
///
/// HTML is the authoring frontend and the reference backend, so it supports the
/// full v0.1 capability set and reports no declared loss for an in-subset scene.
/// Its artifact is byte-identical to [`emit`]; the `target` boundary adds only
/// the capability and loss reporting every other runtime must also provide.
#[derive(Debug, Clone, Copy, Default)]
pub struct HtmlTarget;

impl TargetEmitter for HtmlTarget {
    type Artifact = EmittedScene;

    fn target_id(&self) -> &'static str {
        "html"
    }

    fn capabilities(&self) -> TargetCapabilities {
        TargetCapabilities::all()
    }

    fn emit(&self, scene: &SceneIr) -> TargetEmission<EmittedScene> {
        TargetEmission {
            artifact: emit(scene),
            declared_loss: capability_gaps(scene, &self.capabilities()),
        }
    }
}
