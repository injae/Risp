
mod risp_type;
mod parser;
mod eval;
mod repl;

use crate::repl::repl;

fn main() {
    repl();
}
