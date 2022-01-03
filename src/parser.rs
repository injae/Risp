use std::{num::ParseFloatError, collections::HashMap};
use crate::risp_type::*;
use std::rc::Rc;
use std::convert::TryFrom;
use crate::eval::*;

pub fn tokenize(expr: String) -> Vec<String> {
    expr.replace("'(", " ( list ")
        .replace("(", " ( ")
        .replace(")", " ) ")
        .split_whitespace()
        .map(|x| x.to_string())
        .collect()
}

pub fn parse(tokens: & [String]) -> Result<(RispExp, &[String]), RispErr> {
    let (token, rest) = tokens.split_first()
        .ok_or(RispErr::Reason("could not get token".to_string()))?;

    match &token[..] {
        "(" => read_seq(rest),
        ")" => Err(RispErr::Reason("unexpected `)`".to_string())),
        _   => Ok((parse_atom(token), rest))
    }
} 

fn read_seq(tokens: &[String]) -> Result<(RispExp, &[String]), RispErr> {
    let mut res: Vec<RispExp> = vec![];
    let mut xs = tokens;
    loop {
        let (next_token, rest) = xs
            .split_first()
            .ok_or(RispErr::Reason("could not find closing `)`".to_string()))?;
        if next_token == ")" {
            return Ok((RispExp::List(res), rest));
        }
        let (exp, new_xs) = parse(&xs)?;
        res.push(exp);
        xs = new_xs;
    }
}

fn parse_atom(token: &str) -> RispExp {
    match token.as_ref() {
        "true" => RispExp::Bool(true),
        "false" => RispExp::Bool(false),
        "nil"   => RispExp::Nil,
        _ => {
            let potential_float: Result<f64, ParseFloatError> = token.parse();
            match potential_float {
                Ok(v)  => RispExp::Number(v),
                Err(_) => {
                    if token.starts_with("\"") && token.ends_with("\"") {
                        RispExp::Literal(token[1..token.len()-1].to_string())
                    } else {
                        RispExp::Symbol(token.to_string().clone())
                    }
                },
            }
        }
    }
}

fn parse_single_list(exp: &RispExp) -> Result<Vec<RispExp>, RispErr> {
    match exp {
        RispExp::List(list) => Ok(list.clone()),
        _ => Err(RispErr::Reason("this value is not list".to_string()))
    }
}

//fn parse_list(args: &[RispExp]) -> Result<Vec<RispExp>, RispErr> {
//    Ok(args.to_vec())
//}

fn parse_list_of_floats(args: &[RispExp]) -> Result<Vec<f64>, RispErr> {
    args.iter()
        .map(|x| parse_single_float(x))
        .collect()
}

fn parse_single_float(exp: &RispExp) -> Result<f64, RispErr> {
    match exp {
        RispExp::Number(num) => Ok(*num),
        _ => Err(RispErr::Reason("expected number".to_string()))
    }
}

pub fn parse_list_of_symbol_strings(list: Rc<RispExp>) -> Result<Vec<String>, RispErr> {
    let evaled_list = match list.as_ref() {
        RispExp::List(l) => Ok(l.clone()),
        _ => Err(RispErr::Reason("expected args form to be a list".to_string()))
    }?;
    evaled_list.iter().map(|x| match x {
        RispExp::Symbol(s) => Ok(s.clone()),
        _ => Err(RispErr::Reason("expected symbols in the argument list".to_string()))
    }).collect()
}

macro_rules! inequality_sign {
	($check_fn:expr) => {{
        |args: &[RispExp]| -> RispResult {
            let floats = parse_list_of_floats(args)?;
            let first = floats.first().ok_or(RispErr::Reason("expected at least one number".to_string()))?;
            let rest = &floats[1..];
            fn f (prev: &f64, xs: &[f64]) -> bool {
                match xs.first() {
                    Some(x) => $check_fn(prev, x) && f(x, &xs[1..]),
                    None => true,
                }
            }
            Ok(RispExp::Bool(f(first, rest)))
        }
    }};
}

macro_rules! arithmetic_operation {
    ($math_fn:expr) => {{
        |args: &[RispExp]| -> RispResult {
            let floats = parse_list_of_floats(args)?;
            let result = floats.into_iter().reduce($math_fn).ok_or(RispErr::Reason("expected empty list".to_string()))?;
            Ok(RispExp::Number(result))
        }
    }};
}


pub fn standard_env<'a>() -> RispEnv<'a> {
    let mut data: HashMap<String, RispExp> = HashMap::new();
    data.insert("+".to_string(), RispExp::Func(arithmetic_operation!(|a,b| a + b)));
    data.insert("-".to_string(), RispExp::Func(arithmetic_operation!(|a,b| a - b)));
    data.insert("*".to_string(), RispExp::Func(arithmetic_operation!(|a,b| a * b)));
    data.insert(
        "/".to_string(),
        RispExp::Func(|args: &[RispExp]| -> RispResult {
            let floats = parse_list_of_floats(args)?;
            let first = *floats.first().ok_or(RispErr::Reason("expected at least one number".to_string()))?;
            if first == 0.0 {
                return Err(RispErr::Reason("'/' first element can't 0".to_string()));
            }
            Ok(RispExp::Number(floats[1..].iter().fold(first, |a,b| a / b)))
        }
    ));
    data.insert("=".to_string(), RispExp::Func(inequality_sign!(|a,b| a == b)));
    data.insert(">".to_string(), RispExp::Func(inequality_sign!(|a,b| a > b)));
    data.insert("<".to_string(), RispExp::Func(inequality_sign!(|a,b| a < b)));
    data.insert(">=".to_string(), RispExp::Func(inequality_sign!(|a,b| a >= b)));
    data.insert("<=".to_string(), RispExp::Func(inequality_sign!(|a,b| a <= b)));
    data.insert(
        "list".to_string(),
        RispExp::Func(|args: &[RispExp]| -> RispResult {
            Ok(RispExp::List(args.to_vec()))
        }
    ));
    data.insert(
        "car".to_string(),
        RispExp::Func(|args: &[RispExp]| -> RispResult {
            let [list_exp] = <&[RispExp; 1]>::try_from(args).ok().ok_or(
                RispErr::Reason("Wrong number of arguments: car, 2".to_string())
            )?;
            Ok(native_car(parse_single_list(list_exp)?.as_ref())?)
        }
    ));
    data.insert(
        "cdr".to_string(),
        RispExp::Func(|args: &[RispExp]| -> RispResult {
            let [list_exp] = <&[RispExp; 1]>::try_from(args).ok().ok_or(
                RispErr::Reason("Wrong number of arguments: cdr, 2".to_string())
            )?;
            Ok(native_cdr(parse_single_list(list_exp)?.as_ref())?)
        }
    ));
    data.insert(
        "nth".to_string(),
        RispExp::Func(|args: &[RispExp]| -> RispResult {
            let [idx_exp, list_exp] = <&[RispExp; 2]>::try_from(args).ok().ok_or(
                RispErr::Reason("Wrong number of arguments: nth, 3".to_string())
            )?;
            let list = parse_single_list(list_exp)?;
            let idx = parse_single_float(idx_exp)? as usize;
            Ok(list.into_iter().nth(idx).unwrap_or(RispExp::Nil))
        }
    ));

    RispEnv{data, outer: None}
}

