# Paintbox: Styled Text, Wrapping, and Boxes for CLI Tools

Status: Draft
Owner: RSB/Boxy/Jynx maintainers
Target Crate: paintbox (separate repo, re-exportable from RSB via feature)

## 1. Summary
Paintbox is a small, focused library that provides a common styled-text IR and rendering pipeline for CLI tools. It powers:
- Consistent color/style parsing and theming
- Unicode/emoji-safe width measurement and wrapping
- Terminal box rendering with borders, labels, and padding
- A clean bridge from plain text streams to styled output (ANSI on/off/auto)

Clients: Boxy (decorative boxes), Jynx (syntax highlighting), and any tool that needs robust, themeable CLI output. RSB can optionally re-export Paintbox for ergonomics without owning its complexity.

## 2. Goals
- Share a single style/theming engine across Boxy, Jynx, and other tools.
- Provide a minimal IR for styled text that is easy to compose, test, and render.
- Handle terminal realities: grapheme clustering, emoji, ANSI-safe wrapping, width detection.
- Remain streaming-friendly: convert to/from strings and line-oriented pipelines.
- Keep dependency footprint isolated (feature-gated when re-exported from RSB).

## 3. Non-Goals
- Full TUI framework (windows, widgets, event loops).
- Complex AST for shell commands (remains in RSB domain).
- Owning configuration discovery for every consumer’s CLI flags (Paintbox exposes APIs; tools decide discovery policy).

## 4. Architecture Overview
- IR types: Style, Span, Line, Doc (document as Vec<Line>), BoxStyle/BoxOptions.
- Theme: palette + style classes + token classes, loaded from YAML.
- Renderer: emits ANSI strings (color mode: always/never/auto), or strips ANSI.
- Layout: unicode/emoji-aware measurement and wrapping, optional alignment.
- Bridges: conversion helpers from/to strings and RSB `Stream`.

```
stdin (plain) → Doc (Spans) → [wrap | box] → Renderer (ANSI) → stdout
                                     ↑
                               Theme + Styles
```

## 5. Data Model
- Color: named ("red", "cyan"), 8-bit, 24-bit; foreground/background.
- Attrs: bitflags (bold, dim, italic, underline, blink, inverse, hidden, strike).
- Style: { fg: Option<Color>, bg: Option<Color>, attrs: Attrs }.
- Span: { text: String, style: StyleId } (Style table held by Doc/Renderer).
- Line: Vec<Span>.
- Doc: Vec<Line> + StyleTable (deduplicated styles for compactness).
- Theme: { palette: Map<String, Color>, classes: Map<String, StyleExpr>, tokens: Map<String, StyleExpr> }.
  - StyleExpr: parsed from strings like "cyan,bold on black" with class references.
- BoxStyle: glyph set (plain/round/double), rules for corners, title separators.
- BoxOptions: padding, margin, title (text + style class), width/align policy.

## 6. Configuration & Theming
- YAML schema (initial):
  - palette: { red: "#ff6b6b", cyan: "#2aa198", ... }
  - classes: { info: "cyan,bold", warn: "yellow", error: "red,bold on default" }
  - tokens:  { keyword: "magenta", string: "green", comment: "grey,italic" }
- Loaders: `Theme::from_file(path)`, `Theme::from_yaml(&str)`.
- Merge precedence: built-in defaults < file theme < CLI overrides < env overrides.
- Color mode: `ColorMode::{Always, Never, Auto}` (NO_COLOR respected).

## 7. Rendering & Layout
- Width detection: `tty::width()` with fallback (80 or env).
- Measurement: unicode-grapheme iteration + unicode-width; ignore ANSI when counting.
- Wrapping: greedy wrapping on word boundaries, ANSI-safe span slicing; options for hard/soft wrap.
- Alignment: left/center/right at line level; truncation if requested.
- ANSI: map Style → SGR sequences; ensure proper reset at line boundaries.
- Strip mode: render with all styles removed (for logs/files or when colors disabled).

## 8. Public API (sketch)
Modules:
- `paintbox::theme`
  - `struct Theme;`
  - `impl Theme { fn from_file<P: AsRef<Path>>(p: P) -> Result<Self>; fn resolve_class(&self, name: &str) -> Style; }`
- `paintbox::style`
  - `struct Style { fg: Option<Color>, bg: Option<Color>, attrs: Attrs }`
  - `fn parse(expr: &str, theme: &Theme) -> Result<Style>`
- `paintbox::doc`
  - `struct Doc; struct Span; struct Line;`
  - `impl Doc { fn from_plain(s: &str) -> Self; fn push_span(&mut self, span: Span); }`
- `paintbox::layout`
  - `fn wrap(doc: &Doc, width: usize) -> Doc`
  - `enum Align { Left, Center, Right }`
- `paintbox::render`
  - `struct Renderer { mode: ColorMode }`
  - `impl Renderer { fn render(&self, doc: &Doc) -> String; fn strip(doc: &Doc) -> String; }`
- `paintbox::boxdraw`
  - `struct BoxStyle; struct BoxOptions;`
  - `fn wrap_in_box(doc: &Doc, style: &BoxStyle, opts: &BoxOptions) -> Doc`
- `paintbox::token`
  - `struct TokenRule { pattern: Regex, class: String }`
  - `struct Highlighter { rules: Vec<TokenRule> }`
  - `impl Highlighter { fn highlight(&self, input: &str, theme: &Theme) -> Doc }`

Examples:
```rust
let theme = Theme::from_file("~/.config/paintbox/theme.yaml")?;
let doc = Doc::from_plain(&input);
let wrapped = layout::wrap(&doc, detect_width_or(80));
let boxed = boxdraw::wrap_in_box(&wrapped, &BoxStyle::Rounded, &BoxOptions::title("Build", &theme, "info"));
let out = Renderer::new(ColorMode::Auto).render(&boxed);
print!("{}", out);
```

## 9. Integration with RSB
- Optional feature in RSB: `paintbox` re-export for convenience:
  - `pub use paintbox::{theme, style, doc, layout, render, boxdraw, token};`
- Stream bridges (optional):
  - `impl From<&str> for Doc`, `fn doc_to_stream(doc) -> Stream`, `fn stream_to_doc(stream) -> Doc`.
- RSB macros remain string-first; Paintbox sits as a separate layer for formatting/presentation.

## 10. Integration with Boxy & Jynx
- Boxy: stdin → Doc → wrap → boxdraw → render → stdout; flags select box style, padding, title, theme.
- Jynx: theme + regex rules → highlight(text) → Doc → wrap (optional) → render.
- Both share theme files and style class names; consistent color behavior and NO_COLOR handling.

## 11. Performance & Footprint
- Separate crate to isolate deps (serde_yaml, unicode-segmentation, unicode-width, anstyle/anstream or equivalent).
- Builder API avoids excess allocations; reuse Style table; lazy rendering.
- Fast path: if no colors/wrapping/boxing requested, passthrough as plain.

## 12. Testing Strategy
- Snapshot tests of rendered ANSI for known inputs (with deterministic width).
- Structural tests on Doc/Span operations and wrapping correctness (emoji, wide chars, combining).
- Theme parser tests for precedence and error reporting.
- Fuzzing optional for style parsers.

## 13. Open Questions
- Exact YAML schema and how to reference classes (inheritance/extends?).
- Which ANSI backend (anstyle/anstream vs. manual SGR) for portability.
- Windows quirks (crossterm integration?) and detection strategy.
- How far to go with tokenization DSL vs. pure regex for Jynx.
- Box layout: support multi-column layouts or keep to single-column with padding?

## 14. Milestones
- M1: Core types (Style/Span/Doc), Renderer (ANSI/strip), Theme parser, simple wrap.
- M2: Box drawing (plain/rounded/double), titles, padding/margins.
- M3: Highlighter (regex rules) + token → style mapping.
- M4: RSB feature re-export + Boxy migration; Jynx migration.
- M5: Docs, examples, and theme starter kits.

## 15. Versioning & Compatibility
- Semver; keep Style/Doc stable once Boxy/Jynx migrate.
- Backwards compatibility for theme schema with forward-compatible extensions.

```
rsb (feature = "paintbox") → re-export paintbox modules
boxy → depends on paintbox
jynx → depends on paintbox
```

