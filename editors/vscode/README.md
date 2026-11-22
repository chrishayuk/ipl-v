# IPL-V for VS Code

Editor support for IPL-V decks (`.iplv`): the list-processing language of
the first artificial intelligence programs, as its 1964 manual defines it.
A deck is 80-column cards with a field in fixed columns for each part of a
word, and most of what this extension does is make that comfortable.

- **The card's columns.** Rulers stand at the edges of the fields: comment
  (1-40), TYPE (41), NAME (43-47), SIGN (48), P and Q (49-50), SYMB (51-55)
  and LINK (57-61). **Tab** goes to the next field and **Shift+Tab** to the
  one before, filling a short card out with blanks. The status bar says
  which column and field the cursor is in.
- **What a card says.** Rest the pointer on a word to read it: `70 9-2` is
  "Branch to S if H5 is -. S is 9-2", `J60` is "LOCATE the next symbol after
  cell (0). Sets H5", `W24` is the print line cell. A symbol says where it
  is defined, with that card's comment.
- **Problems as you type.** A comment that runs into column 41, P or Q that
  is not an octal digit, a local symbol used but never defined in its list
  structure, a routine executed that no card defines, a basic process that
  does not exist or is not implemented, a symbol beyond its region, a deck
  with no start card.
- **Symbols.** Go to a symbol's definition, find where it is used, rename
  it in every field it stands in (a local symbol within its own list
  structure), and see the routines and lists of a deck in the outline, each
  with its comment.
- **Colour** for comments, card types, names, prefixes, basic processes,
  system cells, local symbols and data terms, and for the cards behind the
  start card that are the program's to read.
- **Formatting.** Saving a deck leaves it as a keypunch would: upper case,
  nothing after the last punch, each symbol at the left of its field.
- **Run it.** Cmd+Enter (Ctrl+Enter) runs the deck. The command palette
  also has *Run, Tracing Every Instruction* and *Run, with the Assembly
  Listing*. The cards the program reads go behind the start card in the
  same file.
- Snippets for the cards whose columns are always the same: `region`,
  `routines`, `data`, `start`, and `deck` for a whole small one.

## Install

In a clone of the repository:

```
cd editors/vscode
npm install
npm run package
code --install-extension iplv-language-0.1.0.vsix
```

That is all it needs. The extension carries the `iplv` program compiled to
WebAssembly and runs it with the editor's own Node. To use a program you
built yourself, set `iplv.path` in the settings.

Saving a file formats it. To turn that off:

```json
"[iplv]": { "editor.formatOnSave": false }
```

## Building and testing

```
npm install
npm run build        # server/iplv.wasm and dist/extension.js
npm test             # the grammar, the runner and the language server, under Node
npm run test:vscode  # the extension inside a real VS Code
```

`npm run test:vscode` downloads a copy of VS Code the first time, unless
`VSCODE_EXECUTABLE` names one.
