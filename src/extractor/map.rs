//! Mapping functions from resolver/layout types to SEMUI IR types.

use crate::ir::layout::{
    AlignItems, AlignSelf, BoxSizing, Cursor, Display, EdgeInset, FlexDirection, JustifyContent,
    Layout, Position,
};
use crate::ir::paint::{Border, Color, Paint};
use crate::ir::typography::{LineHeight, Typography};
use crate::layout::Geometry;
use crate::resolver::ComputedStyle;

pub fn to_layout(style: &ComputedStyle, geo: &Geometry) -> Layout {
    Layout {
        position: to_position(&style.position),
        display: to_display(&style.display),
        box_sizing: to_box_sizing(&style.box_sizing),
        top: geo.explicit_y,
        left: geo.explicit_x,
        width: geo.width,
        height: geo.height,
        min_width: geo.min_width,
        margin: edge(&geo.margin),
        padding: edge(&geo.padding),
        flex_direction: style.flex_direction.as_deref().and_then(to_flex_direction),
        align_items: style.align_items.as_deref().and_then(to_align_items),
        justify_content: style
            .justify_content
            .as_deref()
            .and_then(to_justify_content),
        align_self: style.align_self.as_deref().and_then(to_align_self),
        gap: style.gap,
    }
}

pub fn to_paint(style: &ComputedStyle) -> Paint {
    let border = if style.border_width > 0.0 {
        style.border_color.as_ref().map(|c| Border {
            width: style.border_width,
            color: Color(c.clone()),
        })
    } else {
        None
    };

    Paint {
        background_color: style.background_color.as_ref().map(|c| Color(c.clone())),
        border,
        border_radius: (style.border_radius > 0.0).then_some(style.border_radius),
        cursor: style.cursor.as_deref().and_then(to_cursor),
    }
}

/// Build typography for a node that has visible text. Returns `None` if any
/// required field (color, font-size, font-weight) was not resolved.
pub fn to_typography(style: &ComputedStyle) -> Option<Typography> {
    Some(Typography {
        font_family: parse_font_family(style.font_family.as_deref()?),
        font_size: style.font_size?,
        font_weight: style.font_weight?,
        line_height: to_line_height(style.line_height.as_deref()),
        color: Color(style.color.clone()?),
    })
}

// ---------------------------------------------------------------------------
// Enum mappers — unknown keywords fall back to CSS initial values
// ---------------------------------------------------------------------------

fn to_position(s: &str) -> Position {
    match s {
        "absolute" => Position::Absolute,
        _ => Position::Static,
    }
}

fn to_display(s: &str) -> Display {
    match s {
        "flex" => Display::Flex,
        "inline-flex" => Display::InlineFlex,
        _ => Display::Block,
    }
}

fn to_box_sizing(s: &str) -> BoxSizing {
    if s == "border-box" {
        BoxSizing::BorderBox
    } else {
        BoxSizing::ContentBox
    }
}

fn to_flex_direction(s: &str) -> Option<FlexDirection> {
    match s {
        "row" => Some(FlexDirection::Row),
        "column" => Some(FlexDirection::Column),
        _ => None,
    }
}

fn to_align_items(s: &str) -> Option<AlignItems> {
    if s == "center" {
        Some(AlignItems::Center)
    } else {
        None
    }
}

fn to_justify_content(s: &str) -> Option<JustifyContent> {
    if s == "center" {
        Some(JustifyContent::Center)
    } else {
        None
    }
}

fn to_align_self(s: &str) -> Option<AlignSelf> {
    match s {
        "auto" => Some(AlignSelf::Auto),
        "flex-start" => Some(AlignSelf::FlexStart),
        _ => None,
    }
}

fn to_cursor(s: &str) -> Option<Cursor> {
    match s {
        "pointer" => Some(Cursor::Pointer),
        "auto" => Some(Cursor::Auto),
        _ => None,
    }
}

fn to_line_height(s: Option<&str>) -> LineHeight {
    match s {
        None | Some("normal") => LineHeight::Normal,
        Some(v) => v
            .trim()
            .strip_suffix("px")
            .and_then(|n| n.trim().parse::<f32>().ok())
            .map(|value| LineHeight::Length { value })
            .unwrap_or(LineHeight::Normal),
    }
}

fn parse_font_family(s: &str) -> Vec<String> {
    s.split(',').map(|f| f.trim().to_owned()).collect()
}

/// Zero-value [`Layout`] for text nodes, which carry no box geometry.
pub fn to_layout_default() -> Layout {
    Layout {
        position: to_position("static"),
        display: to_display("block"),
        box_sizing: to_box_sizing("content-box"),
        top: None,
        left: None,
        width: None,
        height: None,
        min_width: None,
        margin: EdgeInset::zero(),
        padding: EdgeInset::zero(),
        flex_direction: None,
        align_items: None,
        justify_content: None,
        align_self: None,
        gap: None,
    }
}

/// Empty [`Paint`] for text nodes (paint lives on the parent box).
pub fn to_paint_default() -> Paint {
    Paint {
        background_color: None,
        border: None,
        border_radius: None,
        cursor: None,
    }
}

fn edge(sides: &[f32; 4]) -> EdgeInset {
    EdgeInset {
        top: sides[0],
        right: sides[1],
        bottom: sides[2],
        left: sides[3],
    }
}
