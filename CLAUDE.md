# serde_esri

A Cargo workspace with two crates:

- `serde_esri/`: Esri JSON types, `enginex`, `shape`, and the Arrow/GeoArrow conversions.
- `esripbf/`: the prost-generated types for Esri's FeatureCollection PBF, plus conversions into
  `serde_esri` behind its optional `serde_esri` feature. Without that feature it depends only on
  `prost` and `bytes`.

## Rust style

Follow `~/github/rust/rico-root/ricochet/CLAUDE.md` and the rust-development skill: no `unwrap`,
`crate::` paths, no bool parameters, newtypes with `From`/`TryFrom`/`FromStr` instead of free
conversion functions, the narrowest visibility, no `pub use module::*`, no trait objects, one-line
comments and at most two-line `///` docs.

- Struct fields are snake_case. Map Esri's camelCase JSON keys with `#[serde(rename_all = "camelCase")]`
  or `#[serde(rename = "...")]`, never `#[allow(non_snake_case)]`. Only prost-generated code has
  names we don't choose.
- Serde helper attributes go below `#[derive(...)]`; `#[serde_as]` and `#[skip_serializing_none]`
  go above it, or they do nothing.
- Model Esri enumerations as enums, not strings (`Field.field_type: FieldType`,
  `Field.sql_type: Option<SqlType>`), parsed with `Display`/`FromStr` via `serde_with::DisplayFromStr`.
- Never re-export: no `pub use`, `pub(crate) use`, or re-exported dependency crates. Make the
  defining module public and use the item's full path.
- Never annotate `let x: T = ...`. Let inference work, or use a turbofish
  (`serde_json::from_str::<T>()`, `.collect::<Vec<_>>()`). Only prost-generated code is exempt.
- Don't invent wrapper types or helper constructors to route around a design problem; expose the
  real type or express the conversion as `TryFrom` between real types.
- No filler doc comments ("carries", "holds", "calls for"). If a doc adds nothing, especially on
  enum variants, leave it out.
- Name things "type", not "kind".
- Split code into submodules. Tests live in their own files: `foo/tests.rs` declared with
  `#[cfg(test)] mod tests;`, never inline test modules.
- Before editing a module, check that `lib.rs` (or its parent) declares it.

## Working in this repo

- Don't run `cargo fmt` across the workspace; format only the files you changed, and never reformat
  the generated `esripbf/src/esri_p_buffer.rs` (running `rustfmt` on `esripbf/src/lib.rs` reaches it).
- Check with `cargo clippy --workspace --all-features --all-targets` and
  `cargo test --workspace --all-features`.
- Only change things to fix a demonstrated problem: a failing input, a measured cost, or a request.
- No authorship or session trailers in commits or pull requests.
