//! Talks to `iplv lsp` as an editor would, over its standard input and output.

use iplv::json::{self, obj, Json};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Client {
    child: Child,
    to: ChildStdin,
    from: BufReader<ChildStdout>,
    next: usize,
}

const URI: &str = "file:///tmp/f1.iplv";
const F1: &str = include_str!("../examples/manual-f1.iplv");

impl Client {
    fn start() -> Client {
        let mut child = Command::new(env!("CARGO_BIN_EXE_iplv"))
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let to = child.stdin.take().unwrap();
        let from = BufReader::new(child.stdout.take().unwrap());
        Client {
            child,
            to,
            from,
            next: 1,
        }
    }

    fn send(&mut self, message: Json) {
        let mut body = String::new();
        message.write(&mut body);
        write!(self.to, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
        self.to.flush().unwrap();
    }

    fn read(&mut self) -> Json {
        let mut length = 0;
        loop {
            let mut line = String::new();
            assert!(
                self.from.read_line(&mut line).unwrap() > 0,
                "the server closed its output"
            );
            match line.trim_end().split_once(": ") {
                Some(("Content-Length", n)) => length = n.parse().unwrap(),
                _ if line.trim_end().is_empty() => break,
                _ => {}
            }
        }
        let mut body = vec![0; length];
        self.from.read_exact(&mut body).unwrap();
        json::parse(std::str::from_utf8(&body).unwrap()).unwrap()
    }

    fn notify(&mut self, method: &str, params: Json) {
        self.send(obj([
            ("jsonrpc", "2.0".into()),
            ("method", method.into()),
            ("params", params),
        ]));
    }

    fn request(&mut self, method: &str, params: Json) -> Json {
        let id = self.next;
        self.next += 1;
        self.send(obj([
            ("jsonrpc", "2.0".into()),
            ("id", id.into()),
            ("method", method.into()),
            ("params", params),
        ]));
        let reply = self.read();
        assert_eq!(reply.get("id").num(), Some(id), "{:?}", reply);
        reply
    }

    fn at(line: usize, character: usize) -> Json {
        obj([
            ("textDocument", obj([("uri", URI.into())])),
            (
                "position",
                obj([("line", line.into()), ("character", character.into())]),
            ),
        ])
    }

    /// Tell the server the whole text, and take the problems it publishes.
    fn change(&mut self, text: &str) -> Vec<Json> {
        self.notify(
            "textDocument/didChange",
            obj([
                ("textDocument", obj([("uri", URI.into())])),
                ("contentChanges", vec![obj([("text", text.into())])].into()),
            ]),
        );
        self.read().get("params").get("diagnostics").items().to_vec()
    }
}

#[test]
fn an_editor_session() {
    let mut c = Client::start();
    let init = c.request("initialize", obj([("capabilities", obj([]))]));
    let caps = init.get("result").get("capabilities");
    assert_eq!(caps.get("hoverProvider"), &Json::Bool(true));
    assert_eq!(caps.get("textDocumentSync").num(), Some(1));
    assert_eq!(
        init.get("result").get("serverInfo").get("name").str(),
        Some("iplv")
    );
    c.notify("initialized", obj([]));

    // The manual's program has nothing wrong with it.
    c.notify(
        "textDocument/didOpen",
        obj([("textDocument", obj([("uri", URI.into()), ("text", F1.into())]))]),
    );
    let published = c.read();
    assert_eq!(
        published.get("method").str(),
        Some("textDocument/publishDiagnostics")
    );
    assert!(published.get("params").get("diagnostics").items().is_empty());

    // Resting on J60 says what the card does and what J60 is.
    let hover = c.request("textDocument/hover", Client::at(28, 52));
    let shown = hover.get("result").get("contents").get("value").str().unwrap();
    assert!(
        shown.contains("**00 J60** Execute S") && shown.contains("LOCATE the next symbol after cell (0)"),
        "{}",
        shown
    );
    assert_eq!(
        hover
            .get("result")
            .get("range")
            .get("start")
            .get("character")
            .num(),
        Some(50)
    );

    // 9-2 in `70 9-2` leads to the card named 9-2, and is written twice.
    let def = c.request("textDocument/definition", Client::at(29, 51));
    assert_eq!(
        def.get("result").get("range").get("start").get("line").num(),
        Some(36)
    );
    assert_eq!(
        c.request("textDocument/references", Client::at(29, 51))
            .get("result")
            .items()
            .len(),
        2
    );
    assert_eq!(
        c.request("textDocument/documentHighlight", Client::at(29, 51))
            .get("result")
            .items()
            .len(),
        2
    );

    // Renaming F1 rewrites the two fields it stands in, five columns each.
    let mut rename = Client::at(25, 43);
    if let Json::Obj(fields) = &mut rename {
        fields.push(("newName".into(), "F2".into()));
    }
    let renamed = c.request("textDocument/rename", rename);
    let edits = renamed.get("result").get("changes").get(URI).items();
    assert_eq!(edits.len(), 2);
    assert!(edits.iter().all(|e| e.get("newText").str() == Some("F2   ")));

    let outline = c.request("textDocument/documentSymbol", Client::at(0, 0));
    let names: Vec<&str> = outline
        .get("result")
        .items()
        .iter()
        .map(|s| s.get("name").str().unwrap())
        .collect();
    assert_eq!(names, ["E1", "F1", "K1"]);
    assert_eq!(
        outline.get("result").items()[2].get("detail").str(),
        Some("THE LIST K1.")
    );
    let folds = c.request("textDocument/foldingRange", Client::at(0, 0));
    assert_eq!(folds.get("result").items()[1].get("startLine").num(), Some(25));
    assert_eq!(folds.get("result").items()[1].get("endLine").num(), Some(37));

    // The first token is the comment on line 0: thirty-four columns of it.
    let colours = c.request("textDocument/semanticTokens/full", Client::at(0, 0));
    let data: Vec<usize> = colours
        .get("result")
        .get("data")
        .items()
        .iter()
        .map(|n| n.num().unwrap())
        .collect();
    assert_eq!(&data[..5], [0, 0, 34, 0, 0]);
    assert_eq!(data.len() % 5, 0);

    let offered = c.request("textDocument/completion", Client::at(26, 50));
    let items = offered.get("result").items();
    assert!(items.iter().any(|i| i.get("label").str() == Some("J66")
        && i.get("detail")
            .str()
            .unwrap()
            .starts_with("INSERT (0) at the end")));
    assert!(items.iter().any(|i| i.get("label").str() == Some("F1")));

    // A deck already in order is not changed by formatting; one in lower
    // case is.
    assert!(c
        .request("textDocument/formatting", Client::at(0, 0))
        .get("result")
        .items()
        .is_empty());
    assert!(c.change(&F1.to_lowercase()).is_empty());
    let edits = c.request("textDocument/formatting", Client::at(0, 0));
    assert_eq!(edits.get("result").items()[0].get("newText").str(), Some(F1));

    // A change that executes a process that does not exist is an error on
    // that card, at the columns of SYMB.
    let problems = c.change(&F1.replace("J6\n", "J139\n"));
    assert_eq!(problems.len(), 1, "{:?}", problems);
    assert_eq!(problems[0].get("severity").num(), Some(1));
    assert_eq!(problems[0].get("range").get("start").get("line").num(), Some(32));
    assert_eq!(
        problems[0].get("range").get("start").get("character").num(),
        Some(50)
    );
    assert_eq!(
        problems[0].get("message").str(),
        Some("there is no basic process J139")
    );

    let unknown = c.request("textDocument/typeDefinition", Client::at(0, 0));
    assert_eq!(
        unknown.get("error").get("message").str(),
        Some("method not found: textDocument/typeDefinition")
    );

    assert_eq!(c.request("shutdown", Json::Null).get("result"), &Json::Null);
    c.notify("exit", Json::Null);
    assert!(c.child.wait().unwrap().success());
}
