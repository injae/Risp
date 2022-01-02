use crate::risp_type::*;
use crate::parser::*;
use crate::eval::*;

pub fn parse_eval(exp: String, env: &mut RispEnv) -> RispResult {
    let (parsed_exp, _) = parse(&tokenize(exp))?;
    let evaled_exp = eval(&parsed_exp, env)?;
    Ok(evaled_exp)
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
