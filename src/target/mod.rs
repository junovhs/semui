//! Runtime-neutral target-emitter contract (`RET-01`).
//!
//! This module defines the boundary every SEMUI target adapter consumes. Per
//! `DEC-04`, HTML/CSS is the authoring frontend, not the permanent execution
//! target: each emitter must reconstruct a scene from the canonical [`SceneIr`]
//! alone, never by reparsing source HTML/CSS. That invariant is enforced
//! structurally — [`TargetEmitter::emit`] receives only `&SceneIr`, so a target
//! has no access to the source graph.
//!
//! The contract has five parts:
//!
//! 1. [`TargetEmitter`] — the interface a runtime adapter implements.
//! 2. [`Capability`] / [`TargetCapabilities`] — what a scene requires and what a
//!    target supports, so unsupported constructs surface as explicit
//!    [`CapabilityGap`] loss instead of silent drops.
//! 3. [`SceneResources`] — the runtime-neutral resource contract (fonts and
//!    colors) a target must provision.
//! 4. [`Conventions`] — the coordinate and typography conventions the IR uses.
//! 5. [`ConformanceScene`] — the runtime-neutral conformance fixture format that
//!    `RET-03` compares a target's observed output against.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::ir::{
    ControlKind, Display, IrNode, Layout, NodeKind, Paint, Position, SceneIr, Typography,
};

pub mod geometry;
pub mod gpu;

#[cfg(test)]
mod tests;

// ---------------------------------------------------------------------------
// Conventions
// ---------------------------------------------------------------------------

/// The coordinate and typography conventions every target interprets Scene IR
/// under. These are fixed for the v0.1 IR and are part of the contract: a target
/// that uses a different origin or length unit must convert into these before
/// comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conventions {
    /// Length unit for every numeric layout/paint/typography value.
    pub length_unit: String,
    /// Origin of the coordinate space.
    pub origin: String,
    /// Direction of increasing `x`.
    pub x_axis: String,
    /// Direction of increasing `y`.
    pub y_axis: String,
    /// Box model the per-node `box_sizing` field is expressed against.
    pub box_model: String,
    /// How `font_family` is interpreted.
    pub font_fallback: String,
}

impl Conventions {
    /// The v0.1 conventions: CSS pixels, a top-left origin with `y` growing
    /// downward, explicit per-node box-sizing, and ordered font-family fallback.
    pub fn v0_1() -> Self {
        Self {
            length_unit: "css-px".to_string(),
            origin: "top-left".to_string(),
            x_axis: "rightward".to_string(),
            y_axis: "downward".to_string(),
            box_model: "per-node box-sizing".to_string(),
            font_fallback: "ordered font-family stack, first available wins".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Capability model
// ---------------------------------------------------------------------------

/// A discrete visual capability a scene can require and a target can support.
///
/// The set is deliberately coarse: it names the families of behavior a
/// non-browser runtime must implement to reproduce a scene, not every CSS
/// property. A target declares the subset it supports; anything a scene needs
/// beyond that is reported as a [`CapabilityGap`] rather than silently dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Default block flow layout.
    BlockLayout,
    /// Flex / inline-flex layout with the v0.1 flex subset.
    FlexLayout,
    /// `position: absolute` with `top`/`left` offsets.
    AbsolutePositioning,
    /// A solid border on a box.
    Border,
    /// Rounded corners (`border-radius`).
    BorderRadius,
    /// A background fill color.
    Background,
    /// Text rendering with the typography contract.
    Typography,
    /// A native button control.
    ButtonControl,
}

impl Capability {
    /// Every capability defined by the v0.1 contract.
    pub fn all() -> &'static [Capability] {
        use Capability::*;
        &[
            BlockLayout,
            FlexLayout,
            AbsolutePositioning,
            Border,
            BorderRadius,
            Background,
            Typography,
            ButtonControl,
        ]
    }
}

/// The set of capabilities a target supports.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TargetCapabilities {
    supported: BTreeSet<Capability>,
}

impl TargetCapabilities {
    /// A target that supports nothing.
    pub fn none() -> Self {
        Self::default()
    }

    /// A target that supports every capability in the v0.1 contract.
    pub fn all() -> Self {
        Self {
            supported: Capability::all().iter().copied().collect(),
        }
    }

    /// Build from an explicit capability list.
    pub fn from_capabilities(caps: impl IntoIterator<Item = Capability>) -> Self {
        Self {
            supported: caps.into_iter().collect(),
        }
    }

    /// Whether this target supports `capability`.
    pub fn supports(&self, capability: Capability) -> bool {
        self.supported.contains(&capability)
    }
}

/// A single capability a scene requires that the target does not support.
///
/// Targets report these as declared loss so unsupported constructs are explicit,
/// mirroring the diagnostics contract for unsupported source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityGap {
    /// The IR node id that needs the capability.
    pub node_id: String,
    /// The capability the target lacks.
    pub capability: Capability,
}

/// The capabilities `node` requires, in canonical (sorted) order.
fn node_capabilities(node: &IrNode) -> BTreeSet<Capability> {
    let mut caps = BTreeSet::new();
    match node.kind {
        NodeKind::Text => {
            caps.insert(Capability::Typography);
        }
        NodeKind::Box | NodeKind::Control => {
            match node.layout.display {
                Display::Block => {
                    caps.insert(Capability::BlockLayout);
                }
                Display::Flex | Display::InlineFlex => {
                    caps.insert(Capability::FlexLayout);
                }
            }
            if node.layout.position == Position::Absolute {
                caps.insert(Capability::AbsolutePositioning);
            }
            if node.paint.background_color.is_some() {
                caps.insert(Capability::Background);
            }
            if node.paint.border.is_some() {
                caps.insert(Capability::Border);
            }
            if node.paint.border_radius.is_some() {
                caps.insert(Capability::BorderRadius);
            }
            if node.typography.is_some() {
                caps.insert(Capability::Typography);
            }
            if matches!(node.control_kind, Some(ControlKind::Button)) {
                caps.insert(Capability::ButtonControl);
            }
        }
    }
    caps
}

/// Every capability the `scene` requires across all nodes, in canonical order.
pub fn scene_capabilities(scene: &SceneIr) -> BTreeSet<Capability> {
    scene.nodes.iter().flat_map(node_capabilities).collect()
}

/// The declared loss for emitting `scene` to a target with `capabilities`:
/// one [`CapabilityGap`] per (node, unsupported capability), in pre-order by
/// node and canonical order by capability.
pub fn capability_gaps(scene: &SceneIr, capabilities: &TargetCapabilities) -> Vec<CapabilityGap> {
    let mut gaps = Vec::new();
    for node in preorder(scene) {
        for capability in node_capabilities(node) {
            if !capabilities.supports(capability) {
                gaps.push(CapabilityGap {
                    node_id: node.id.clone(),
                    capability,
                });
            }
        }
    }
    gaps
}

// ---------------------------------------------------------------------------
// Resource contract
// ---------------------------------------------------------------------------

/// A font a scene requires: an ordered fallback stack and the weights used with
/// it. A target provisions fonts by satisfying the stack in order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontRequest {
    /// Ordered family fallback list, e.g. `["Inter", "sans-serif"]`.
    pub family_stack: Vec<String>,
    /// Distinct integer weights requested for this stack, ascending.
    pub weights: Vec<u16>,
}

/// The runtime-neutral resources a scene needs a target to provision.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneResources {
    /// Distinct font stacks with their requested weights, ordered by first use.
    pub fonts: Vec<FontRequest>,
    /// Distinct colors referenced anywhere in the scene, ascending.
    pub colors: Vec<String>,
}

/// Collect the [`SceneResources`] a target must provision to reproduce `scene`.
pub fn collect_resources(scene: &SceneIr) -> SceneResources {
    let mut fonts: Vec<(Vec<String>, BTreeSet<u16>)> = Vec::new();
    let mut colors: BTreeSet<String> = BTreeSet::new();

    for node in preorder(scene) {
        if let Some(color) = &node.paint.background_color {
            colors.insert(color.0.clone());
        }
        if let Some(border) = &node.paint.border {
            colors.insert(border.color.0.clone());
        }
        if let Some(typography) = &node.typography {
            colors.insert(typography.color.0.clone());
            record_font(&mut fonts, typography);
        }
    }

    SceneResources {
        fonts: fonts
            .into_iter()
            .map(|(family_stack, weights)| FontRequest {
                family_stack,
                weights: weights.into_iter().collect(),
            })
            .collect(),
        colors: colors.into_iter().collect(),
    }
}

/// Record a typography node's font stack and weight, merging weights into an
/// existing stack so each distinct fallback list appears once.
fn record_font(fonts: &mut Vec<(Vec<String>, BTreeSet<u16>)>, typography: &Typography) {
    match fonts
        .iter_mut()
        .find(|(stack, _)| *stack == typography.font_family)
    {
        Some((_, weights)) => {
            weights.insert(typography.font_weight);
        }
        None => {
            let mut weights = BTreeSet::new();
            weights.insert(typography.font_weight);
            fonts.push((typography.font_family.clone(), weights));
        }
    }
}

// ---------------------------------------------------------------------------
// Traversal
// ---------------------------------------------------------------------------

/// Deterministic pre-order traversal of a scene's node tree: a parent precedes
/// its children, and siblings keep source order. Every target visits nodes in
/// this order so output ordering is contract-defined rather than target-defined.
pub fn preorder(scene: &SceneIr) -> Vec<&IrNode> {
    let mut out = Vec::with_capacity(scene.nodes.len());
    visit_children(&scene.nodes, None, &mut out);
    out
}

fn visit_children<'a>(nodes: &'a [IrNode], parent_id: Option<&str>, out: &mut Vec<&'a IrNode>) {
    for node in nodes
        .iter()
        .filter(|node| node.parent_id.as_deref() == parent_id)
    {
        out.push(node);
        visit_children(nodes, Some(&node.id), out);
    }
}

// ---------------------------------------------------------------------------
// Target emitter interface
// ---------------------------------------------------------------------------

/// A target adapter that reconstructs a scene from the canonical IR.
///
/// Implementors receive only `&SceneIr`; per `DEC-04` they must never reparse
/// source HTML/CSS. An emission carries both the produced artifact and the
/// explicit [`CapabilityGap`] loss the target could not reproduce.
pub trait TargetEmitter {
    /// The artifact this target produces (e.g. an HTML document, a draw list).
    type Artifact;

    /// Stable identifier for this target, e.g. `"html"`.
    fn target_id(&self) -> &'static str;

    /// The capabilities this target supports.
    fn capabilities(&self) -> TargetCapabilities;

    /// Reconstruct `scene` into this target's artifact. Must be deterministic:
    /// the same IR always yields the same emission.
    fn emit(&self, scene: &SceneIr) -> TargetEmission<Self::Artifact>;
}

/// The result of a target emission: the artifact plus declared loss.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetEmission<A> {
    /// The target-specific output.
    pub artifact: A,
    /// Capabilities the scene required that this target could not honor.
    pub declared_loss: Vec<CapabilityGap>,
}

// ---------------------------------------------------------------------------
// Cross-runtime conformance fixture format
// ---------------------------------------------------------------------------

/// The runtime-neutral observable contract for a single node: everything a
/// target must reproduce, with source provenance (`SourceRef`) intentionally
/// dropped. This is what `RET-03` compares a target's observed render against.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformanceNode {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub kind: NodeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control_kind: Option<ControlKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
    pub layout: Layout,
    pub paint: Paint,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typography: Option<Typography>,
}

/// The runtime-neutral conformance fixture for a scene: the conventions, the
/// required capabilities and resources, and the per-node observable contract in
/// pre-order. Serializing this yields the stable cross-runtime fixture format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformanceScene {
    pub scene_id: String,
    pub corpus: String,
    pub conventions: Conventions,
    pub required_capabilities: Vec<Capability>,
    pub resources: SceneResources,
    pub nodes: Vec<ConformanceNode>,
}

/// Derive the expected [`ConformanceScene`] from canonical IR. This is the
/// neutral expectation each target is measured against; it strips provenance and
/// fixes node order to [`preorder`].
pub fn expected_conformance(scene: &SceneIr) -> ConformanceScene {
    ConformanceScene {
        scene_id: scene.scene_id.clone(),
        corpus: scene.corpus.clone(),
        conventions: Conventions::v0_1(),
        required_capabilities: scene_capabilities(scene).into_iter().collect(),
        resources: collect_resources(scene),
        nodes: preorder(scene)
            .into_iter()
            .map(|node| ConformanceNode {
                id: node.id.clone(),
                parent_id: node.parent_id.clone(),
                kind: node.kind.clone(),
                control_kind: node.control_kind.clone(),
                text_content: node.text_content.clone(),
                layout: node.layout.clone(),
                paint: node.paint.clone(),
                typography: node.typography.clone(),
            })
            .collect(),
    }
}
