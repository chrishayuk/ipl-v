//! A language server for IPL-V decks, speaking the Language Server Protocol
//! over standard input and output. `iplv lsp` runs it.
//!
//! It keeps the text of each open file and answers from `check.rs`: problems
//! as the text changes, what a card says and what the symbol under the
//! pointer is, where a symbol is defined and used, the outline of routines
//! and lists, completion of the basic processes, colours by column, and a
//! formatter that leaves the deck as a keypunch would have.

use crate::check;
use crate::json::{self, obj, Json};
use crate::names;
use std::collections::HashMap;
use std::io::{BufRead, Write};

fn read_message(input: &mut impl BufRead) -> Option<Json> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if input.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                length = value.trim().parse::<usize>().ok();
            }
        }
    }
    let mut body = vec![0; length?];
    input.read_exact(&mut body).ok()?;
    json::parse(&String::from_utf8_lossy(&body)).ok()
}

fn send(output: &mut impl Write, message: Json) {
    let mut body = String::new();
    message.write(&mut body);
    let _ = write!(output, "Content-Length: {}\r\n\r\n{}", body.len(), body);
    let _ = output.flush();
}

fn position(line: usize, character: usize) -> Json {
    obj([("line", line.into()), ("character", character.into())])
}

fn range(line: usize, start: usize, end: usize) -> Json {
    obj([("start", position(line, start)), ("end", position(line, end))])
}

fn diagnostics(uri: &str, text: &str) -> Json {
    let items: Vec<Json> = check::check(text)
        .into_iter()
        .map(|d| {
            obj([
                ("range", range(d.line - 1, d.start, d.end)),
                ("severity", if d.error { 1 } else { 2 }.into()),
                ("source", "iplv".into()),
                ("message", d.message.into()),
            ])
        })
        .collect();
    obj([
        ("jsonrpc", "2.0".into()),
        ("method", "textDocument/publishDiagnostics".into()),
        (
            "params",
            obj([("uri", uri.into()), ("diagnostics", items.into())]),
        ),
    ])
}

fn capabilities() -> Json {
    obj([
        (
            "capabilities",
            obj([
                // 1: the whole text is sent on every change.
                ("textDocumentSync", 1.into()),
                ("hoverProvider", true.into()),
                ("definitionProvider", true.into()),
                ("referencesProvider", true.into()),
                ("documentSymbolProvider", true.into()),
                ("documentFormattingProvider", true.into()),
                ("documentHighlightProvider", true.into()),
                ("renameProvider", true.into()),
                ("foldingRangeProvider", true.into()),
                (
                    "semanticTokensProvider",
                    obj([
                        (
                            "legend",
                            obj([
                                (
                                    "tokenTypes",
                                    check::TOKEN_TYPES
                                        .iter()
                                        .map(|t| Json::from(*t))
                                        .collect::<Vec<_>>()
                                        .into(),
                                ),
                                ("tokenModifiers", Vec::new().into()),
                            ]),
                        ),
                        ("full", true.into()),
                    ]),
                ),
                ("completionProvider", obj([("resolveProvider", false.into())])),
            ]),
        ),
        (
            "serverInfo",
            obj([
                ("name", "iplv".into()),
                ("version", env!("CARGO_PKG_VERSION").into()),
            ]),
        ),
    ])
}

fn completion(text: &str) -> Json {
    // The basic processes, and the names the deck defines.
    let mut items: Vec<Json> = (0..210u32)
        .filter_map(|n| names::process(n).map(|p| (n, p)))
        .map(|(n, p)| {
            obj([
                ("label", format!("J{}", n).into()),
                ("kind", 3.into()), // Function
                ("detail", p.into()),
                ("sortText", format!("1{:03}", n).into()),
            ])
        })
        .collect();
    for (name, routine, comment, line, _) in check::outline(text) {
        items.push(obj([
            ("label", name.into()),
            ("kind", if routine { 3 } else { 6 }.into()), // Function, Variable
            (
                "detail",
                if comment.is_empty() {
                    format!("line {}", line + 1)
                } else {
                    comment
                }
                .into(),
            ),
            ("sortText", format!("0{:05}", line).into()),
        ]));
    }
    items.into()
}

fn symbols(text: &str) -> Json {
    let width = |line: usize| {
        text.lines()
            .nth(line)
            .map(|l| l.encode_utf16().count())
            .unwrap_or(0)
    };
    check::outline(text)
        .into_iter()
        .map(|(name, routine, comment, first, last)| {
            obj([
                ("name", name.into()),
                ("detail", comment.into()),
                ("kind", if routine { 12 } else { 13 }.into()), // Function, Variable
                (
                    "range",
                    obj([
                        ("start", position(first, 0)),
                        ("end", position(last, width(last))),
                    ]),
                ),
                ("selectionRange", range(first, 42, 47)),
            ])
        })
        .collect::<Vec<_>>()
        .into()
}

/// Tokens as the protocol wants them: five numbers each, positions relative
/// to the token before.
fn semantic_tokens(text: &str) -> Json {
    let mut data = Vec::new();
    let (mut last_line, mut last_start) = (0, 0);
    for (line, start, length, kind) in check::semantic(text) {
        let delta = if line == last_line {
            start - last_start
        } else {
            start
        };
        data.extend([line - last_line, delta, length, kind, 0].map(Json::from));
        (last_line, last_start) = (line, start);
    }
    obj([("data", data.into())])
}

fn formatting(text: &str) -> Json {
    let new = check::format(text);
    if new == text {
        return Json::Arr(Vec::new());
    }
    // One edit that replaces the whole text.
    let lines = text.split('\n').count();
    let last = text.rsplit('\n').next().unwrap_or("").encode_utf16().count();
    vec![obj([
        (
            "range",
            obj([("start", position(0, 0)), ("end", position(lines - 1, last))]),
        ),
        ("newText", new.into()),
    ])]
    .into()
}

/// Answer one request. `None` means the method is not one this server knows.
fn answer(method: &str, params: &Json, docs: &HashMap<String, String>) -> Option<Json> {
    let uri = params.get("textDocument").get("uri").str().unwrap_or("");
    let text = docs.get(uri).map(String::as_str).unwrap_or("");
    let line = params.get("position").get("line").num().unwrap_or(0);
    let col = params.get("position").get("character").num().unwrap_or(0);
    let location = |(l, a, b): check::Place| obj([("uri", uri.into()), ("range", range(l, a, b))]);
    Some(match method {
        "initialize" => capabilities(),
        "shutdown" => Json::Null,
        "textDocument/hover" => match check::hover(text, line, col) {
            Some((value, (l, a, b))) => obj([
                (
                    "contents",
                    obj([("kind", "markdown".into()), ("value", value.into())]),
                ),
                ("range", range(l, a, b)),
            ]),
            None => Json::Null,
        },
        "textDocument/documentHighlight" => check::references(text, line, col)
            .into_iter()
            .map(|(l, a, b)| obj([("range", range(l, a, b))]))
            .collect::<Vec<_>>()
            .into(),
        "textDocument/rename" => {
            match check::renamed(text, line, col, params.get("newName").str().unwrap_or("")) {
                Some(edits) => {
                    let edits: Vec<Json> = edits
                        .into_iter()
                        .map(|((l, a, b), new)| obj([("range", range(l, a, b)), ("newText", new.into())]))
                        .collect();
                    obj([("changes", Json::Obj(vec![(uri.to_string(), edits.into())]))])
                }
                None => Json::Null,
            }
        }
        "textDocument/foldingRange" => check::outline(text)
            .into_iter()
            .filter(|(.., first, last)| last > first)
            .map(|(.., first, last)| obj([("startLine", first.into()), ("endLine", last.into())]))
            .collect::<Vec<_>>()
            .into(),
        "textDocument/semanticTokens/full" => semantic_tokens(text),
        "textDocument/definition" => check::definition(text, line, col)
            .map(location)
            .unwrap_or(Json::Null),
        "textDocument/references" => check::references(text, line, col)
            .into_iter()
            .map(location)
            .collect::<Vec<_>>()
            .into(),
        "textDocument/documentSymbol" => symbols(text),
        "textDocument/completion" => completion(text),
        "textDocument/formatting" => formatting(text),
        _ => return None,
    })
}

/// Serve until the client says `exit` or closes the input.
pub fn serve(input: &mut impl BufRead, output: &mut impl Write) -> Result<(), String> {
    let mut docs: HashMap<String, String> = HashMap::new();
    let mut shutting_down = false;
    while let Some(message) = read_message(input) {
        let method = message.get("method").str().unwrap_or("").to_string();
        let params = message.get("params");
        let id = message.get("id").clone();
        let uri = params
            .get("textDocument")
            .get("uri")
            .str()
            .unwrap_or("")
            .to_string();
        match method.as_str() {
            "exit" => {
                return if shutting_down {
                    Ok(())
                } else {
                    Err("exit without shutdown".into())
                }
            }
            "textDocument/didOpen" => {
                let text = params
                    .get("textDocument")
                    .get("text")
                    .str()
                    .unwrap_or("")
                    .to_string();
                send(output, diagnostics(&uri, &text));
                docs.insert(uri, text);
            }
            "textDocument/didChange" => {
                if let Some(text) = params
                    .get("contentChanges")
                    .items()
                    .last()
                    .and_then(|c| c.get("text").str())
                {
                    send(output, diagnostics(&uri, text));
                    docs.insert(uri, text.to_string());
                }
            }
            "textDocument/didClose" => {
                docs.remove(&uri);
                send(output, diagnostics(&uri, ""));
            }
            // A notification this server has no use for.
            _ if id == Json::Null => {}
            _ => {
                shutting_down |= method == "shutdown";
                let reply = match answer(&method, params, &docs) {
                    Some(result) => obj([("jsonrpc", "2.0".into()), ("id", id), ("result", result)]),
                    None => obj([
                        ("jsonrpc", "2.0".into()),
                        ("id", id),
                        (
                            "error",
                            obj([
                                ("code", Json::Num(-32601.0)),
                                ("message", format!("method not found: {}", method).into()),
                            ]),
                        ),
                    ]),
                };
                send(output, reply);
            }
        }
    }
    Ok(())
}
