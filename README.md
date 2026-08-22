# Ea

Entity access. Stores and exposes entities uniformly. Ea also happens to be the
Akkadian name of the Mesopotamian god of wisdom, clever counsel, and practical
knowledge.

`ea` reads and writes individual settings inside configuration that it does not
own, across many storage kinds and file formats, without disturbing anything it
was not asked to change. What it can enumerate, read and write is the `entity`,
which it accesses thusly.

```sh
ea get alacritty.font.mono              # read 11 for example
ea set alacritty.font.mono 13           # write 13 to that path
ea dump > ~/.config/ea/profiles/frame   # capture the whole host
ea apply ~/.config/ea/profiles/frame    # reproduce it elsewhere
```

## The problem

Configuration lives in a dozen incompatible places. Some of it is a text file
you wrote by hand, some is a text file the application rewrites whenever you
touch its preferences dialog, and some is not a file at all and needs an
external tool to reach.

Editing it from a script means regular expressions against formatting, which is
brittle: when whatever you're matching on stops matching, nothing happens and
nothing complains. A silent no-op is indistinguishable from success, and equally
indistinguishable from an application that was skipped because it is not
installed.

Editing it with a parser is worse, because parsing and reserialising throws away
comments, blank lines, key order and quoting style. That is unacceptable for
files a human also edits, and possibly destructive for files the application
rewrites.

## The model

There are essentially six core concepts to `ea`:

  * Storage: where and how is something stored
  * Format:  what is the format of the storage
  * Locator: how is the key addressed within that storage
  * Codec:   how is a value we care about encoded
  * Key:     the `ea` name of a value container
  * Value:   a value to be encoded

### Storage

How to read and write the bytes. Two kinds to begin with:

  * `file`: a path on disk.
  * `tool`: an external tool that gets and sets, such as `gsettings`.

A third kind, binary stores reached through a library rather than a tool, is
anticipated but not implemented. The trait exists so that adding it does not
disturb anything else.

Storage answers only "give me the bytes" and "take these bytes back". It does
not know what format they are in.

### Format

What those bytes are, and therefore how they can be addressed.

A format names a tree-sitter grammar. Given bytes it produces a concrete syntax
tree, and given a tree plus a set of edits it splices them back into bytes.
TOML, Lua and elisp to begin with.

Format is the hinge between storage and locator. Storage does not know what it
is holding, and a locator cannot be written without knowing which grammar it is
querying, so the coupling between them belongs to the format. Supporting a new
kind of file means registering a grammar.

Not every storage has a format. A `tool` storage hands back a single value
rather than a document, so there is nothing to parse and the locator addresses
the tool directly.

### Locator

Where, within some storage, the value sits.

For files this is a tree-sitter query over a named grammar, with a capture
called `@value` marking the node whose bytes are the setting. Optional
constraints on other captures narrow the match, which is how one query serves
every host or every table without being rewritten.

For tools it is whatever addressing the tool itself uses, such as a gsettings
schema plus key.

Note that the locator is bound to the *format*, not to the storage. TOML, Lua
and XML need entirely different locators and are all "file", while the same TOML
locator should work whether those bytes came from a local path, over a network,
or out of an archive. Keeping these independent is what makes a push model
possible later.

### Codec

How a value is encoded at some locator within some storage.

The size is rarely a bare number. It is usually a field inside a larger string
that has its own microsyntax, and that microsyntax belongs to the *setting*, not
to the file format:

```text
11                                              bare
Sans Bold 10                                    pango
Monospace:h11.2:#e-subpixelantialias:#h-none    vim guifont
Monospace,11,-1,5,50,0,0,0,0,0                  qt font list
-*-helvetica-bold-r-*-*-34-*-*-*-*-*-iso8859-15 xlfd, size is field 8
10.5pt                                          css length
```

A grammar for the containing file will never help here. tree-sitter hands you
the Lua string as a single `string_content` node; what is inside it is not Lua.
Codecs are hand-written, and they are the long tail of this project. Where
possible they should be data-driven, since a delimiter plus a field index covers
XLFD, the qt list and pango.

### Key

The public, stable name of a setting. `alacritty.font.mono`.

This is what profiles record and what the command line accepts. It is
deliberately independent of where the value actually lives, so that an
application moving its config file, or changing its config format, does not
invalidate every profile ever written. The key is an interface; the locator
behind it is an implementation detail.

### Value

The decoded form of whatever sits at a locator.

A codec turns the bytes at a locator into a value, and a value back into bytes.
A value is structured rather than scalar: a font descriptor has a family, a
size, and often weights or flags that `ea` has no opinion about.

Still open: whether a key addresses a whole descriptor or a single field of one.
`gnome.font.sans` could mean `Sans 10.5`, with `gnome.font.sans.size` addressing
`10.5` beneath it, or it could address the size directly and keep the descriptor
internal. The first generalises better and makes `describe` richer; the second
is simpler and is what the examples above assume.

## Losslessness

The rule applies at three levels:

1. **File**: a concrete syntax tree preserves comments, whitespace, ordering and
   quoting.
2. **Value**: a codec preserves fields it was not asked to change, and
   round-trips fields it does not understand.
3. **Write**: a byte-range splice preserves the rest of the file.

For example, a codec written as "parse to a number, format it back" will discard
`:#e-subpixelantialias` the first time it touches a neovim config, and it will
look like it worked.

## Setting a value is merging

You never set a font size. You take an existing descriptor, change one field,
and put back everything else exactly as it was. `Sans Bold 10` must keep `Sans
Bold`. The XLFD font descriptor of a `.Xresources` file must keep thirteen of
its fourteen fields as-is.

So a codec decodes to a structured descriptor, not to, say, a simple number or
string. `set` is a field update on that descriptor followed by re-encoding.

## Tree-sitter

The obvious idea is to describe each file format with a grammar and let the
program work out how to change a value. That idea is right, but, for example, a
plain EBNF describes recognition, but not necessarily serialization. Editing
needs to serialize back with everything you did not touch intact.

An inspiration for this is [Augeas](https://augeas.net/), which has done roughly
this since the 2000s, using bidirectional "lenses" to expose config files as
trees.

[tree-sitter](https://tree-sitter.github.io/) provides parsers for a myriad of
file formats. It produces a [concrete syntax
tree](https://en.wikipedia.org/wiki/Parse_tree). Unlike an [abstract syntax
tree](https://en.wikipedia.org/wiki/Abstract_syntax_tree), a concrete syntax
tree keeps every byte, including comments, blank lines and the original quoting.
That is what makes round-tripping possible from an ordinary, unidirectional
grammar, with no "lens" required. The grammars are maintained by the tree-sitter
ecosystem, independently of `ea`.

The idea of `ea` is thus, more or less, `Augeas` but written in Rust, using
`tree-sitter` to do as much of the heavy lifting as possible.

## Findings that shaped the design of Ea

**Grammars vary enormously in how much they tell you.** For example, TOML is
typed and pleasant. The parse tree for a `[font]` table containing `size = 11`:

```text
(document (table (bare_key) (pair (bare_key) (integer))))
```

elisp is not. It is a generic tree of lists and symbols with no fields and no
semantics, and `:size` and `14` are merely adjacent siblings:

```text
(source_file (list (symbol) (list (symbol) (list (symbol)) (string))
  (special_form (symbol) (list (symbol) (symbol) (string) (symbol)
  (integer) (symbol) (quote (symbol))) ...)))
```

A locator therefore cannot be a simple dotted path. It has to be a full query
with predicates and anchors. It remains data, which is the point of `ea`, but it
is not always short/easy data.

**Cross products are problematic.** A query matching a `(symbol)` and a `(list)`
under the same parent matches every combination of them. Against a real `emacs`
config that produced twelve matches where eight were correct, silently pairing,
say, `doom-font` with the variable-pitch size. The fix is a `.` anchor, which
requires immediate "siblinghood". Any query selecting sibling pairs needs one.

**Enumeration falls out for free.** The same elisp query finds all four host
blocks without being told they exist which is what makes `describe` and `dump`
possible. Prefer query locators over line anchors wherever there is a choice,
because anchors cannot enumerate.

**Constraints belong outside the query.** Selecting the current host is done by
constraining the `@host` capture at match time, and not by, say templating a
hostname into the query string.

## Verbs

  * `get <key>` reads one value.
  * `set <key> <value>` writes one value.
  * `dump` reads every value this host exposes, producing a profile.
  * `describe` lists what keys exist at all, whether or not the application is
    installed (i.e. schema, not data).
  * `apply <profile>` sets many values at once.

`dump` and `apply` are inverses. A profile is `key = value` lines, which is
hand-editable, greppable and diffs cleanly:

```text
alacritty.font.mono = 11
neovim.font.mono = 11.2
gnome.font.sans = 10.5
```

Profiles are named collections of keys and values which can be applied by `ea`
to configuration (storage) on a host.

The name is just a label: a host, a screen, a location, a time of day, whatever
distinguishes one set of values from another.

## Plan and apply

Every edit is computed and validated before any of them is written, so a profile
that cannot apply in full fails before a single byte changes. That covers what
usually goes wrong: a locator matching nothing, a missing file, a value the
codec rejects.

It is not a rollback guarantee. Once writing has begun a failure can still leave
a host partly changed, and for `tool` storage that cannot be avoided, since
there is nothing to abort once `gsettings` has returned.

  * **Batch by document.** Group edits per file, parse once, apply all, write
    once. Does not read-parse-write per setting.
  * **Byte offsets invalidate each other.** Splice the first edit and every
    later offset in that file is stale. Apply in descending offset order, or
    feed edits back through tree-sitter's incremental API. This fails by
    corrupting a file rather than by erroring, so it deserves a test of its own.

## Errors

There are no silent errors; errors are always explicit and clearly communicated.
In particular, silent errors are a bug.

A locator that matches nothing is an error, and it names the key, the storage
and the query. An application that is not installed is a separate condition,
reported separately.

## Layout

```text
src/
  main.rs        cli entry, clap subcommands
  storage/       file and tool backends
  format/        grammar registry, parse and splice
  locator/       query compilation, capture constraints
  codec/         value microsyntax, decode and encode
  module/        setting definitions, one per key
  plan.rs        edit computation, batching, ordering
```

An `ea` module is data, not code. It names a key, a storage, a format, a locator
and a codec. Adding an application should not require new logic, and once the
set is stable, module definitions will move out of Rust and into files so that
adding new modules does not require a rebuild.

In the main concepts overview, we also noted `key` and `value`; it is not yet
clear if those should become sub-directories under source also.

## Status

Alpha. Currently scaffolding and design validation only.

Proof-of-concept modules:

| Module    | Storage | Format | Codec       | Proves                         |
| --------- | ------- | ------ | ----------- | ------------------------------ |
| alacritty | file    | TOML   | bare        | happy path, table scoping      |
| neovim    | file    | Lua    | vim guifont | losslessness inside the value  |
| emacs     | file    | elisp  | bare        | locators in an untyped grammar |
| gnome     | tool    | n/a    | pango       | using a tool instead of a file |

## Non-goals

  * Owning configuration files. `ea` edits what exists; laying files down in the
    first place is a different job.
  * Converting between formats.

## Requirements

Rust 1.85 or newer, which is where edition 2024 landed. Development happens on
1.98.

## License

Copyright © 2026 Rubin Simons

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License, version 3, as published by
the Free Software Foundation.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program. If not, see <https://www.gnu.org/licenses/>.

Version 3 only. The option to use a later version is deliberately not granted.
See `LICENSE` for the full text.
