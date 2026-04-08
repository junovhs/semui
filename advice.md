You could get **surprisingly far** by wiring together existing Rust dependencies.

A plausible v0 stack is:

- **HTML parsing / DOM tree:** `html5ever` directly, or `scraper` / `kuchiki` on top of it. `scraper` already gives browser-grade HTML parsing plus CSS-selector querying via Servo’s `html5ever` and `selectors` crates. ([Docs.rs][1])
- **CSS parsing:** `lightningcss`. It fully parses CSS rules, properties, and values into typed normalized structures, built on Mozilla’s `cssparser` and `selectors` foundations. ([Docs.rs][2])
- **Selector matching:** either use the selector support already exposed through `scraper`, or use the lower-level selector machinery indirectly through the Servo/Mozilla stack that Lightning CSS and scraper rely on. ([Docs.rs][1])
- **Layout engine:** `taffy` for block/flex/grid computation. It already implements CSS Block, Flexbox, and Grid layout algorithms and is explicitly meant to power UI trees. ([Docs.rs][3])
- **SVG normalization/render support:** `usvg` / `resvg` if you need icons or SVG assets normalized or rasterized. `usvg` is specifically a simplification layer that resolves SVG complexity into a strongly typed tree, and `resvg` renders that tree. ([Docs.rs][4])

So with dependencies alone, you can likely assemble this:

```text
HTML -> DOM tree
CSS -> typed stylesheet
Selectors -> matched rules per node
Styles -> merged declarations
Layout -> computed boxes for many common cases
SVG -> normalized asset handling
Emit -> SEMUI
```

That is enough for a **real v0**.

What existing deps likely get you **without heroic invention**:

- Parse HTML robustly. ([Docs.rs][1])
- Parse most CSS into structured values. ([Docs.rs][2])
- Query elements with selectors. ([Docs.rs][1])
- Compute a lot of static layout if you map your DOM/style tree into Taffy nodes carefully. ([Docs.rs][3])
- Normalize SVG content for icons/assets. ([Docs.rs][4])

That means the parts you **must build** are not the obvious parsing parts. The hard stuff is the glue and the semantics.

## What you must build yourself

### 1. DOM + CSS -> **computed style resolver**

This is the first real proprietary layer.

You need logic for:

- cascade priority
- inheritance
- default expansion
- shorthand expansion
- variable resolution strategy
- final per-node computed values

Libraries can parse CSS, but **your system must decide what “resolved enough for SEMUI” means**. That is core product logic.

### 2. **SEMUI schema / IR**

No dependency will define your actual visual semantic language for you.

You need to choose:

- node model
- component vs node vs variant boundaries
- state representation
- token representation
- layout vocabulary
- paint vocabulary
- output serialization format

This is probably the most important thing you own.

### 3. **Component / variant extraction**

This is a very “you” problem.

Dependencies can parse and compute. They will not tell you:

- these 7 buttons are one base component
- these 3 cards are variants
- this repeated DOM shape should become a semantic abstraction

That abstraction inference is one of the main reasons SEMUI exists.

### 4. **Browser-to-SEMUI normalization policy**

Even if you can parse and compute everything, you still need to decide:

- what level of truth to preserve
- what to flatten
- what to keep semantic
- what to keep as tokens
- what to infer as reusable structure

That normalization policy is the actual product.

### 5. **State / interaction extraction**

Dependencies won’t cleanly solve:

- hover / active / selected state modeling
- JS event semantics
- local UI state extraction
- modal open/close patterns
- list binding / interaction contracts

For v0 you can dodge most of this, but eventually this becomes one of the hardest parts.

### 6. **Target emitters**

No dependency gives you:

- SEMUI -> SwiftUI
- SEMUI -> WGPU UI
- SEMUI -> Dioxus
- SEMUI -> Compose

That compiler layer is yours.

## Where I think you can get the furthest fastest

If I were scoping this to maximize momentum, I’d do:

### Use deps for:

- HTML parse
- CSS parse
- selector application
- partial layout
- SVG handling

### Build yourself:

- computed style merge layer
- SEMUI IR
- static HTML/CSS -> SEMUI emitter
- SEMUI -> HTML/CSS round-trip emitter
- simple repeated-structure detector

That gets you to the first real proof:

```text
HTML/CSS -> SEMUI -> HTML/CSS
```

If that round-trip is visually stable, you’re in business.

## The hardest thing technically

Not parsing. Not even layout.

The hardest thing is probably:

> **deciding the boundary between “resolved visual truth” and “semantic reusable abstraction.”**

Because those goals fight each other.

- Too resolved, and SEMUI becomes a giant flattened dump.
- Too semantic, and you lose fidelity.

That balancing act is the real invention.

## My blunt answer

You can probably get **60–75% of a compelling static v0** by stringing together mature dependencies. The remaining **25–40%** is the actual moat:

- computed-style normalization
- SEMUI IR design
- abstraction extraction
- cross-runtime emitters
- bounded interaction/state semantics

That’s good news, because it means:

- the plumbing is available
- the hard part left is exactly the differentiated part

So yes: **dependency stack for the substrate, custom code for the semantics.**

A very practical v0 dependency stack would be:

```toml
html5ever / scraper or kuchiki
lightningcss
taffy
usvg / resvg
serde
```

And the first custom modules I’d build are:

```text
style_resolver
layout_bridge
semui_ir
semui_emit
component_infer
```

That’s the shortest path to something real.

[1]: https://docs.rs/scraper/latest/scraper/?utm_source=chatgpt.com "scraper - Rust - Docs.rs"
[2]: https://docs.rs/lightningcss/latest/lightningcss/?utm_source=chatgpt.com "lightningcss - Rust - Docs.rs"
[3]: https://docs.rs/taffy/latest/taffy/?utm_source=chatgpt.com "taffy - Rust - Docs.rs"
[4]: https://docs.rs/resvg/latest/resvg/?utm_source=chatgpt.com "resvg - Rust - Docs.rs"
