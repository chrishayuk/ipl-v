//! The session of `debug.rs` as a WebAssembly module, for `web/`. Built
//! with `cargo build --release --target wasm32-unknown-unknown`.
//!
//! Text goes in through a buffer the page fills (`iplv_buf`) and comes out
//! through one it reads (`iplv_result`, after a call that returns a
//! length). Storage itself is read where it lies: `iplv_mem` is the address
//! of the first cell, and each cell is twelve bytes.

use crate::debug::{Mode, Session};
use std::cell::RefCell;

thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
    static BUF: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static RESULT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn give(text: String) -> u32 {
    RESULT.with(|r| {
        *r.borrow_mut() = text.into_bytes();
        r.borrow().len() as u32
    })
}

fn with<T>(default: T, f: impl FnOnce(&mut Session) -> T) -> T {
    SESSION.with(|s| match s.borrow_mut().as_mut() {
        Some(s) => f(s),
        None => default,
    })
}

fn buf_text(from: usize, to: usize) -> String {
    BUF.with(|b| String::from_utf8_lossy(&b.borrow()[from..to]).into_owned())
}

/// A buffer of `len` bytes for the page to write text into.
#[no_mangle]
pub extern "C" fn iplv_buf(len: u32) -> *mut u8 {
    BUF.with(|b| {
        let mut b = b.borrow_mut();
        b.clear();
        b.resize(len as usize, 0);
        b.as_mut_ptr()
    })
}

#[no_mangle]
pub extern "C" fn iplv_result() -> *const u8 {
    RESULT.with(|r| r.borrow().as_ptr())
}

/// Load the deck in the first `deck` bytes of the buffer, with the next
/// `input` bytes as the cards behind it. Returns 1 if it loaded.
#[no_mangle]
pub extern "C" fn iplv_load(deck: u32, input: u32, cells: u32) -> u32 {
    let (deck, input) = (deck as usize, input as usize);
    let session = Session::new(&buf_text(0, deck), &buf_text(deck, deck + input), cells as usize);
    let ok = session.error.is_none();
    SESSION.with(|s| *s.borrow_mut() = Some(session));
    ok as u32
}

/// Interpret up to `n` instructions. `mode` is 0 for just that, 1 to stop
/// when the present routine goes on, 2 when it has finished. Returns 0 if
/// it stopped for that, 1 halted, 2 on an error, 3 at a breakpoint.
#[no_mangle]
pub extern "C" fn iplv_advance(n: u32, mode: u32) -> u32 {
    let mode = match mode {
        1 => Mode::Over,
        2 => Mode::Out,
        _ => Mode::Steps,
    };
    with(1, |s| s.advance(n as u64, mode) as u32)
}

#[no_mangle]
pub extern "C" fn iplv_state() -> u32 {
    give(with(String::from("null"), |s| s.state()))
}

#[no_mangle]
pub extern "C" fn iplv_lines() -> u32 {
    give(with(String::from("[]"), |s| s.lines()))
}

#[no_mangle]
pub extern "C" fn iplv_structure(root: u32, limit: u32) -> u32 {
    give(with(String::from("null"), |s| s.structure(root, limit as usize)))
}

#[no_mangle]
pub extern "C" fn iplv_describe(a: u32) -> u32 {
    give(with(String::new(), |s| s.describe(a)))
}

/// The address of the symbol written in the first `len` bytes of the
/// buffer, or -1.
#[no_mangle]
pub extern "C" fn iplv_resolve(len: u32) -> i32 {
    let text = buf_text(0, len as usize);
    with(-1, |s| s.resolve(&text).map(|a| a as i32).unwrap_or(-1))
}

/// Take what has been printed (0), traced (1) or warned of (2) since the
/// last time.
#[no_mangle]
pub extern "C" fn iplv_take(what: u32) -> u32 {
    give(with(String::new(), |s| match what {
        0 => std::mem::take(&mut s.m.out),
        1 => std::mem::take(&mut s.m.trace_out),
        _ => s.m.warnings.drain(..).map(|w| w + "\n").collect(),
    }))
}

#[no_mangle]
pub extern "C" fn iplv_break(a: u32, on: u32) {
    with((), |s| {
        if on != 0 {
            s.breakpoints.insert(a);
        } else {
            s.breakpoints.remove(&a);
        }
    })
}

/// 0: trace what the program's marks say; 1: everything; 2: nothing.
#[no_mangle]
pub extern "C" fn iplv_trace(mode: u32) {
    with((), |s| {
        s.m.trace_all = match mode {
            1 => Some(true),
            2 => Some(false),
            _ => None,
        }
    })
}

#[no_mangle]
pub extern "C" fn iplv_mem() -> *const crate::word::Word {
    with(std::ptr::null(), |s| s.m.mem.as_ptr())
}

#[no_mangle]
pub extern "C" fn iplv_mem_len() -> u32 {
    with(0, |s| s.m.mem.len() as u32)
}
