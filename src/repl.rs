use crate::risp_type::*;
use crate::parser::*;
use crate::eval::*;

pub fn parse_eval(exp: String, env: &mut RispEnv) -> RispResult {
    let mut token = tokenize(exp);
    loop {
        let (parsed_exp, remain) = parse(&token)?;
        token = remain.to_vec();
        if token.is_empty() {
            return Ok(eval(&parsed_exp, env)?)
        } else {
            eval(&parsed_exp, env)?;
        }
    }
}

fn slurp_expr() -> String {
    let mut expr = String::new();
    std::io::stdin().read_line(&mut expr)
        .expect("Failed to read line");
    expr
}

pub fn repl() {
    let env = &mut standard_env();
    loop {
        print!("risp >\n");
        let expr = slurp_expr();
        match parse_eval(expr, env) {
            Ok(res) => println!("// 🔥 => {}", res),
            Err(e) => match e {
                RispErr::Reason(msg) => println!("// 🙀 => {}", msg),
            },
        }
    }
}
