//! IPL-V, Information Processing Language V, as an IPL computer: the cell
//! storage, loader, interpreter and basic processes of the *IPL-V Manual*
//! (Newell and others; second edition, 1964).

pub mod auxiliary;
pub mod card;
pub mod check;
pub mod debug;
pub mod interp;
pub mod json;
pub mod loader;
pub mod lsp;
pub mod machine;
pub mod names;
pub mod prims;
pub mod print;
pub mod restart;
pub mod tape;
#[cfg(target_arch = "wasm32")]
pub mod web;
pub mod word;

pub use interp::Stop;
pub use machine::Machine;

/// The total available space, in cells, unless asked otherwise: what the
/// manual's own run had on a 32,768-word machine (Part One, Fig. 1).
pub const DEFAULT_CELLS: usize = 22_000;
pub const DEFAULT_CYCLES: u64 = 100_000_000;

/// Load a deck and run it from its start card.
pub fn run(m: &mut Machine, limit: u64) -> Result<Stop, String> {
    match m.load()? {
        Some(loader::Start::At(start)) => {
            m.start(start);
            m.run(limit)
        }
        Some(loader::Start::Restart) => m.run(limit),
        None => Err("the deck has no start card (TYPE 5 with a routine in SYMB)".to_string()),
    }
}
