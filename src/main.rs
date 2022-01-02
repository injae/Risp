
mod risp_type;
mod parser;
mod eval;
mod repl;

use crate::repl::repl;

fn main() {
    println!("{}",vec![0.0, 1., 2.].into_iter().reduce(|a,b| a + b).expect(""));
    repl();
}
