use crate::eval::*;
use crate::parser::*;
use crate::risp_type::*;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::{Cmd, CompletionType, Config, Context, EditMode, Editor, KeyEvent};
use rustyline_derive::Helper;

use std::borrow::Cow::{self, Borrowed, Owned};

pub fn parse_eval(exp: String, env: &mut RispEnv) -> RispResult {
    let mut token = tokenize(exp);
    loop {
        let (parsed_exp, remain) = parse(&token)?;
        token = remain.to_vec();
        if token.is_empty() {
            return Ok(eval(&parsed_exp, env)?);
        } else {
            eval(&parsed_exp, env)?;
        }
    }
}

fn slurp_expr() -> String {
    let mut expr = String::new();
    std::io::stdin()
        .read_line(&mut expr)
        .expect("Failed to read line");
    expr
}

//pub fn repl() {
//    let env = &mut standard_env();
//    loop {
//        print!("risp >\n");
//        let expr = slurp_expr();
//        match parse_eval(expr, env) {
//            Ok(res) => println!("// 🔥 => {}", res),
//            Err(e) => match e {
//                RispErr::Reason(msg) => println!("// 🙀 => {}", msg),
//            },
//        }
//    }
//}

#[derive(Helper)]
pub struct RispHelper {
    pub completer: FilenameCompleter,
    pub highlighter: MatchingBracketHighlighter,
    pub validator: MatchingBracketValidator,
    pub hinter: HistoryHinter,
    pub colored_prompt: String,
}

impl Completer for RispHelper {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        self.completer.complete(line, pos, ctx)
    }

    fn update(&self, line: &mut rustyline::line_buffer::LineBuffer, start: usize, elected: &str) {
        let end = line.pos();
        line.replace(start..end, elected)
    }
}

impl Hinter for RispHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for RispHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for RispHelper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}
