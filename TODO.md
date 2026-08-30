# Todo

Goal: build `ea` to working state. Each step is one component, and each
component practices specific Rust skills. Order: vertical slice first
(get + set for alacritty/TOML end to end), then breadth (formats, tool
storage, profiles, external module data).

## Rules (every step)

- `cargo fmt` and `cargo clippy -- -D warnings` clean before review
- unit tests in-file under `#[cfg(test)] mod tests`
- no new crates without agreement (`tempfile` agreed; serde deferred to step 10)
- a step is done when the review says it is

## References

- [Rust for Rustaceans](https://nostarch.com/rust-rustaceans) (Jon Gjengset,
  2024); matches this stage; skim the trait and lifetime chapters when needed
- [The Rust Reference](https://doc.rust-lang.org/reference/) for syntax
  questions
- [tree-sitter Rust bindings](https://docs.rs/tree-sitter/latest/tree_sitter/)
  for the parser/query API; [query
  reference](https://tree-sitter.github.io/tree-sitter/using-parsers/queries)
  for S-expressions
- Grammar query files for S-expression examples, local cargo registry:
  [tree-sitter-toml-ng](file:///home/rubin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tree-sitter-toml-ng-0.7.0/queries/),
  [tree-sitter-lua](file:///home/rubin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tree-sitter-lua-0.5.0/queries/),
  [tree-sitter-elisp](file:///home/rubin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tree-sitter-elisp-1.7.2/queries/)

## Steps

### 1. Scaffolding: modules + anyhow context

Flat files for now (README's `storage/` dirs are the end state; split a file
into a dir only when it exceeds ~300 lines):

```text
main.rs     cli (clap) + dispatch only, no logic
storage.rs  Storage trait + FileStorage
format.rs   Format enum, parse
locator.rs  query compile + execute
codec.rs    codecs + value types
plan.rs     edit computation (appears at step 6)
modules.rs  key -> module data (appears at step 7, externalized at 10)
```

**Decided:** `anyhow` everywhere. The lesson is the `Context` idiom: every
failure site that owes the README's "name the key/storage/query" gets a
`.with_context(|| format!(...))` so the message carries it. If a typed
error becomes genuinely needed (distinguishing "not installed" from "no
match" in describe, perhaps), we add it then.

**Tasks:**
- create the stub modules, `mod storage;` ... declared in main.rs
- give each command body its own function, `fn cmd_get(key: &str)` etc.,
  still `todo!()` inside
- context drill: a `fn read(p: &Path) -> anyhow::Result<Vec<u8>>` in
  storage.rs that wraps `std::fs::read` so the error message names the
  path; a test asserts a failed read's message contains the path

**Skills:** modules and `use` paths, `?` propagation, `anyhow::Context`
(closure form is lazy: the message is built only on failure).

**Done:** skeleton compiles, context test passes, clippy -D warnings clean.
**Status:** in-progress

### 2. Storage: file backend

**Tasks:**
- `trait Storage { fn read(&self) -> Result<Vec<u8>>; fn write(&self, &[u8]) -> Result<()>; }`
- `FileStorage { path: PathBuf }` via `std::fs::read`/`write`
- errors carry the `path`
- tests with `tempfile::tempdir()`: round-trip temp file; missing path
  yields an error naming the path

**Skills:** traits and impl blocks, `Path`/`PathBuf` idioms, `?` on
`io::Error`, where `Vec<u8>` ownership goes.

**Done:** both tests pass.
**Status:** not-started

### 3. Format: parsing (TOML only, for now)

**Tasks:**
- `enum Format { Toml, Lua, Elisp }` with `parse(bytes) -> Result<Tree>`
- find in the tree-sitter docs: `Parser::new()`,
  `parser.set_language(&tree_sitter_toml_ng::language())?`,
  `parser.parse(bytes, None) -> Option<Tree>`
- inspect by hand: `tree.root_node().to_sexp()`
- note: `node.utf8_text(bytes)` returns `Option<&str>` (non-UTF-8 bytes)

**Skills:** FFI-flavored API, `&[u8]` vs `Vec<u8>`, `Node` borrowing from the
`Tree`, Option from fallible conversion.

**Done:** you can point at the integer node of a font size in a sample
alacritty.toml by sexp alone.
**Status:** not-started

### 4. Locator: query compile + execute

**Tasks:**
- locator is data: an S-expression string with an `@value` capture
- find in docs: `Query::new(text, &lang)`, `QueryCursor::execute(...)`,
  iterate matches, find the `@value` capture id via
  `query.capture_name_for_id(idx)`
- constraint = second capture (e.g. `@host`) filtered in the loop,
  NOT templated into the query text (README: constraints live outside
  the query)
- return `node.byte_range()` of the value node; zero matches is an error
  that names key + query

**Skills:** owned iterators, matching `Option<&str>`, working in `usize`
byte ranges.

**Done:** a pure function `locate(bytes, format, locator) -> byte range`
returns the right range on the sample; tested.
**Status:** not-started

### 5. Codec + first end-to-end `get` (vertical slice complete)

**Tasks:**
- value type starts as an enum with a `Bare(String)` arm, room for
  structured arms later
- `Pango` codec: `Sans Bold 10` -> `{ family, weight, size, extras }`;
  unknown pieces must round-trip (README losslessness level 2)
- the idiomatic dance: `impl FromStr for Pango { type Err = ...; }`
- wire the CLI: `ea get alacritty.font.mono` = read file -> parse ->
  locate -> decode -> print

**Skills:** `str::split`/`split_once`, `FromStr` with typed error,
`Display`, round-trip test tables (`encode(decode(s)) == s` over real
examples).

**Done:** `ea get alacritty.font.mono` prints `11` on your machine;
Pango round-trips a table of ~10 real examples.
**Status:** not-started

### 6. Plan + apply: `ea set`

**Tasks:**
- `plan.rs`: pure function `(key, value) -> Vec<Edit>` where
  `Edit { offset, old: Range<usize>, new: Vec<u8> }`; no IO, so it
  tests directly
- batch edits per storage (`HashMap<PathBuf, Vec<Edit>>`), sort
  descending by offset, splice into `Vec<u8>` via `split_at` +
  `extend_from_slice`
- validate by reparsing the spliced bytes
- `--dry-run` shows the resulting bytes without writing

**Skills:** `sort_by_key` + `rev`, slice arithmetic, the pure-core /
IO-shell split.

**Done:** `ea set alacritty.font.mono 13` changes only that byte range;
diff against the original shows nothing else moved (comments and blank
lines intact).
**Status:** not-started

### 7. dump / describe / apply (profiles)

**Tasks:**
- dump: every module, get each key, print one `key = value` line; key
  absent on this host = design choice: skip vs stderr warning
  (discuss before coding)
- describe: enumerates the schema from module data only, no storage
- apply: parse a profile file into `Vec<(key, value)>`, run it through
  step 6's batcher

**Skills:** `HashMap` vs `BTreeMap` (deterministic output), `BufRead` line
iteration, stdout vs stderr discipline.

**Done:** `ea dump > p; ea apply p --dry-run` on the same host plans zero
changes.
**Status:** not-started

### 8. More formats: Lua (guifont) + Elisp (the hard one)

**Tasks:**
- neovim guifont codec: `Monospace:h11.2:#e...:#h-none`, colon fields
- emacs/elisp: untyped grammars; the README cross-product finding bites
  here (adjacent `(symbol)`/`(list)` captures pair up cartesian, fix is
  the `.` sibling anchor)

**Skills:** same machinery against grammar quirks; reading tree-sitter
elisp output patiently.

**Done:** `ea set` round-trips a real `init.lua` and `init.el` with
comments intact.
**Status:** not-started

### 9. Tool storage: gsettings + pango

**Tasks:**
- `ToolStorage` behind the same `Storage` trait: `std::process::Command`,
  `.arg(...)`, `.output()`
- classify failures: spawn failure (tool missing) vs non-zero exit
  (key unknown vs tool complaint; read stderr) vs other

**Skills:** `Command`, `Output`, `ExitStatus`, error classification,
  blocking process IO.

**Done:** `ea get/set gnome.font.sans` works on a live GNOME session.
**Status:** not-started

### 10. Module data out of the crate

**Tasks:**
- move `modules.rs` data into `mod/` files (README layout), loaded at
  startup; `include_str!` for v1, or serde_json if we want structured
  (this is where a serde dep earns its keep)
- remaining codecs (xlfd, css length) and modules as needed

**Skills:** `include_str!`, serde derive if chosen, startup vs baked-in
config.

**Done:** adding a module changes no compiled code.
**Status:** not-started

## Calibration (resolved)

- Shaky spots: none in particular
- Error style: B, anyhow everywhere; context strings carry key/path/query
- tempfile: agreed
