# Ea

Everything anything: work with k/v pairs in many file formats. Ea is also the
Akkadian name of the Mesopotamian god of wisdom, counsel, and practical
knowledge.

Configuration comes in many different forma. `ea` reads and writes individual
settings inside files and through commands. It works across many storage kinds 
and file formats. It changes only the values you tell it to change.

What `ea` can enumerate, read, and write is the key/value pair (`k/v`). You 
access a `k/v` like this:

```sh
ea get alacritty.font.mono              # read 11 for example
ea set alacritty.font.mono 13           # write 13 to that path
ea dump > ~/.config/ea/profiles/frame   # capture the whole host
ea apply ~/.config/ea/profiles/frame    # reproduce it elsewhere
```

## The problem

Configuration lives in many incompatible places. Some of it is a text file you
wrote by hand. Some of it is a text file the application rewrites each time
you use its preferences dialog. Some of it is not a file at all, and you need
an external tool to reach it.

Editing it from a script means regular expressions against formatting. This is
brittle. When your pattern stops matching, or worse, starts matching more than
one thing, chaos ensues.

Editing it with a parser has its own problems. Parsing and reserialising can
discard comments, blank lines, ordering, and quoting style. That is bad for
files a human also edits. For files an application manages, it can destroy
data.

## The model

`ea` has six core concepts:

  * Storage: where and how something is stored
  * Format:  what the format of the storage is
  * Locator: how the key is addressed within that storage
  * Codec:   how a value we care about is encoded
  * Key:     the `ea` name of a value container
  * Value:   a value to be encoded

### Storage

Storage reads and writes the bytes. There are two kinds to start with:

  * `file`: a path on disk.
  * `tool`: an external tool that gets and sets, such as `gsettings`.

A third kind, `binary`, is anticipated but not implemented. It would reach
stores through a library instead of a tool.

Storage answers only "give me the bytes" and "take these bytes back". It does
not know what format the bytes are in.

### Format

Format says what those bytes are, and so how you can address them.

A format names a tree-sitter grammar. Given bytes, it produces a concrete
syntax tree. Given a tree plus a set of edits, it splices the edits back into
bytes. To start with: TOML, Lua, and Elisp.

Format is the connection between storage and locator. Storage does not know
what it holds. You cannot write a locator without knowing which grammar it
queries. To support a new kind of file, you register a grammar.

Not every storage has a format. A `tool` storage hands back a single value.
There is nothing to parse, and the locator addresses the tool directly.

### Locator

A locator says where, within some storage, the value sits.

For files, a locator is a tree-sitter query over a named grammar. It has a
capture called `@value` that marks the node whose bytes are the setting.

Optional constraints on a capture can narrow the match. For example, match
only the font setting within the block where `name=Foo`.

For tools, a locator is whatever addressing the tool itself uses. For
`gsettings`, that is a schema plus a key.

The locator is bound to the *format*, not to the storage. TOML, Lua, and XML
need entirely different locators, and they are all of type "file". The same
TOML locator works when the bytes come from a local path, from a network, or
from an archive.

### Codec

A codec says how a value is encoded at a locator within a storage.

A value is often not a scalar. It can be a field inside a larger string that
has its own microsyntax. That microsyntax belongs to the *setting*, not to the
file format:

| Value                                             | Microsyntax           |
| ------------------------------------------------- | --------------------- |
| `11`                                              | bare                  |
| `Sans Bold 10`                                    | pango                 |
| `Monospace:h11.2:#e-subpixelantialias:#h-none`    | vim guifont           |
| `Monospace,11,-1,5,50,0,0,0,0,0`                  | qt font list          |
| `-*-helvetica-bold-r-*-*-34-*-*-*-*-*-iso8859-15` | xlfd, size is field 8 |
| `10.5pt`                                          | css length            |

A grammar for the containing file does not help here. tree-sitter gives you
the Lua string as a single `string_content` node. What is inside it is not
Lua.

### Key

The name of a setting, for example: `alacritty.font.mono`.

A profile contains keys. You can also pass keys on the command line. A key is
independent of where the value lives. An application can move its config file,
or change its config format, without invalidating a profile.

### Value

The decoded form of whatever sits at a locator.

A codec turns the bytes at a locator into a value, and a value back into
bytes. A value is usually structured. It is not a simple scalar. For example,
a font descriptor has a family, a size, and often weights or flags.

Still undecided: whether a key addresses a whole descriptor or a single field
of one. `gnome.font.sans` could mean `Sans 10.5`, with `gnome.font.sans.size`
addressing `10.5` beneath it. Or it could address the size directly and keep
the descriptor internal. The first option generalises better and makes
`describe` richer. The second is simpler, and the examples above assume it.

## Losslessness

Applies at three levels:

1. **File**: a concrete syntax tree keeps comments, whitespace, ordering, and
   quoting.
2. **Value**: a codec decodes all fields, changes only the field you set,
   and encodes the other fields back byte-for-byte.
3. **Write**: `ea` replaces only the byte range of the value you set. All
   other bytes in the file stay as they are.

For example, take a codec written as "parse to a number, format it back". The
first time it touches a neovim config, it will discard `:#e-subpixelantialias`,
and it will look like it worked. The correct codec avoids that.

## Setting a value is merging

Often, you do not set a font size. You take an existing descriptor that has
some microsyntax, change one field, and put back everything else exactly as it
was. Suppose you want to set a font size to `10`, but the setting is stored
as `Sans Bold 10`. Then you must keep `Sans Bold`.

So a codec (microsyntax) decodes to a structured descriptor. It does not
decode to a simple scalar value or string. `set` is a field update on that
descriptor, followed by re-encoding.

## Tree-sitter

The idea is to describe each file format with a grammar, and let the program
work out how to change a value. But a plain EBNF, for example, describes 
recognition, and not serialization. Editing needs to serialize back with 
everything you did not touch intact.

An inspiration for this is [Augeas](https://augeas.net/). Augeas has done
roughly this since the 2000s, using bidirectional "lenses" to expose config
files as trees.

[tree-sitter](https://tree-sitter.github.io/) provides parsers for a myriad of
file formats. It produces a
[concrete syntax tree](https://en.wikipedia.org/wiki/Parse_tree). An
[abstract syntax tree](https://en.wikipedia.org/wiki/Abstract_syntax_tree)
discards bytes. A concrete syntax tree keeps every byte: comments, blank
lines, and the original quoting.

The idea of `ea` is thus, more or less, Augeas written in Rust, using
tree-sitter to do as much of the heavy lifting as possible.

## Findings that influenced the design of Ea

**Grammars vary enormously in how rich they are.** For example, TOML is typed
and pleasant. The parse tree for a `[font]` table containing `size = 11`:

```text
(document (table (bare_key) (pair (bare_key) (integer))))
```

Elisp (at least the tree-sitter parser for it) is not. It is a generic tree
of lists and symbols with no fields and no semantics. `:size` and `14` are
only adjacent siblings:

```text
(source_file (list (symbol) (list (symbol) (list (symbol)) (string))
  (special_form (symbol) (list (symbol) (symbol) (string) (symbol)
  (integer) (symbol) (quote (symbol))) ...)))
```

So a locator cannot be a simple dotted path. It has to be a full query with
support for predicates and anchors. It stays data, which is the point of
`ea`. But it is not always short or easy data.

**Cross products are problematic.** A query matching a `(symbol)` and a
`(list)` under the same parent matches every combination of them. Against a
real `emacs` config, that produced twelve matches where eight were correct.
It silently paired, say, `doom-font` with the variable-pitch size. The fix is
a `.` anchor, which requires immediate "siblinghood". Any query selecting
sibling pairs needs one.

**Enumeration comes for free with tree-sitter.** The same Elisp query finds
all four host blocks without being told they exist. This is what makes the
`describe` and `dump` sub-commands possible. Note: prefer query locators over
line anchors where you have a choice, because anchors cannot enumerate.

**Constraints belong outside the query.** To select the current host,
constrain the `@host` capture at match time. Do not, for example, template a
hostname into the query string.

## Verbs

  * `get <key>` reads one value.
  * `set <key> <value>` writes one value.
  * `dump` reads every value this host exposes, producing a profile.
  * `describe` lists which keys exist at all, whether the application is
    installed or not (schema, not data).
  * `apply <profile>` sets many values at once.

`dump` and `apply` are inverses. A profile is a collection of `key = value`
lines. You can edit it by hand, grep it, and diff it:

```text
alacritty.font.mono = 11
neovim.font.mono = 11.2
gnome.font.sans = 10.5
```

A profile is a named collection of keys and values. `ea` can apply it to
configuration (storage) on a host.

The name is a label: a host, a screen, a location, a time of day, or whatever
else distinguishes one set of values from another.

## Plan and apply

Every edit is computed and validated before any of it is written. A profile
that cannot apply in full fails before anything changes.

This is probably not a rollback guarantee. Once writing has begun, a failure
can still leave a file partly changed. For `tool` storage, you cannot avoid
this. There is nothing to abort once, for example, `gsettings` has returned.

  * **Batch by storage.** Group edits per file or tool, parse once, apply all,
    write once. Do not read-parse-write per setting.
  * **Byte offsets invalidate each other.** Splice the first edit, and every
    later offset in that file is stale. Apply edits in descending offset
    order, or feed them back through tree-sitter's incremental API.

## Errors

Errors are always explicit and clearly communicated. A silent error is a bug.

A locator that matches nothing is an error. It names the key, the storage,
and the query. An application that is not installed is a separate condition,
reported separately.

## Layout

```text
mod/             ea modules (data not code)
src/             rust source code
  main.rs        cli entry, clap subcommands
  storage/       file and tool backends
  format/        grammar registry, parse and splice
  locator/       query compilation, capture constraints
  codec/         value microsyntax, decode and encode
  plan.rs        edit computation, batching, ordering
```

An `ea` module is data. It names a key, a storage, a format, a locator, and a
codec. Adding an application must not require new logic. Once the set is stable,
module definitions will move out of Rust and into files. Then adding new modules
will not require a rebuild.

The main concepts overview also lists `key` and `value`. It is not yet clear
whether those become sub-directories under source as well.

## Status

Alpha. Currently scaffolding and design validation only.

Proof-of-concept modules:

| Module    | Storage | Format | Codec       | Proves                         |
| --------- | ------- | ------ | ----------- | ------------------------------ |
| alacritty | file    | TOML   | bare        | happy path, table scoping      |
| neovim    | file    | Lua    | vim guifont | losslessness inside the value  |
| emacs     | file    | Elisp  | bare        | locators in an untyped grammar |
| gnome     | tool    | n/a    | pango       | using a tool instead of a file |

## Non-goals

  * Owning configuration files. `ea` edits what exists. Laying files down in
    the first place is a different job.
  * Converting between formats.

## Requirements

Rust 1.85 or newer (edition 2024). Development happens on latest stable.

## License

Copyright © 2026 Rubin Simons

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License, version 3, as published
by the Free Software Foundation.

This program is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more
details.

You should have received a copy of the GNU Affero General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.

Version 3 only. The option to use a later version is deliberately not
granted. See `LICENSE` for the full text.
