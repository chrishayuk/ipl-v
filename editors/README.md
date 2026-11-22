# IPL-V in your editor

Two pieces give an editor an understanding of a deck:

- **`iplv lsp`**, a language server. It is a command of the `iplv` program
  itself, so there is nothing else to build, and it works with any editor
  that speaks the Language Server Protocol.
- **A Visual Studio Code extension** in [`vscode`](vscode), which carries
  the program and adds the card's columns (rulers, and Tab from field to
  field), highlighting, snippets and run commands. Its
  [README](vscode/README.md) lists the features.

Both are built on [`src/check.rs`](../src/check.rs), the same code behind
`iplv check` and `iplv format`.

## The language server

In a clone of this repository, `cargo build --release`. The server is then
`target/release/iplv lsp`. It talks on standard input and output.

It provides diagnostics, hover, go to definition, references, document
highlights, rename, completion of the basic processes and of the deck's
own names, document symbols, folding ranges, formatting and semantic
tokens. Semantic tokens mean an editor with no grammar for cards still gets
highlighting, and they know what a grammar cannot: whether a card is in a
routine or in data, and which cards are behind the start card.

### Neovim (0.11 or later)

```lua
vim.filetype.add({ extension = { iplv = "iplv" } })
vim.lsp.config("iplv", {
  cmd = { "/path/to/iplv", "lsp" },
  filetypes = { "iplv" },
})
vim.lsp.enable("iplv")
```

### Helix

In `languages.toml`:

```toml
[language-server.iplv-lsp]
command = "/path/to/iplv"
args = ["lsp"]

[[language]]
name = "iplv"
scope = "source.iplv"
file-types = ["iplv"]
language-servers = ["iplv-lsp"]
```

These two configurations follow each editor's documented format but have
not been run here. The server itself is tested over standard input and
output by `tests/lsp.rs` and `vscode/test/server.test.js`, and inside VS
Code by `vscode/test/suite.js`.

In either editor, set a column ruler at 40, 42, 47, 50, 55, 56 and 61 to
see the fields.

## Checking a deck from the command line

```sh
iplv check deck.iplv
```

prints one line per problem, as `file:line:column: kind: message`, and
exits with status 1 if any is an error. `iplv format deck.iplv` prints the
deck in upper case with each symbol at the left of its field. Both take `-`
for standard input.
