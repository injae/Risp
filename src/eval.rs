use crate::risp_type::*;
use crate::parser::*;
use std::collections::HashMap;
use std::rc::Rc;
use std::convert::TryFrom;
use std::fs;
use crate::repl::parse_eval;

fn eval_built_in_func(exp: &RispExp, args: &[RispExp], env: &mut RispEnv) -> Option<RispResult> {
    match exp {
        RispExp::Symbol(s) => {
            match s.as_ref() {
                "if"  => Some(eval_if_arg(args, env)),
                "let" => Some(eval_let_arg(args, env)),
                "fn" => Some(eval_lambda_arg(args)),
                "load" => Some(eval_load_risp_file(args, env)),
                "env" => Some(eval_print_env(env)),
                "print" => Some(eval_print(args, env)),
                _ => None,
            }
        }
        _ => None
    }
}

fn eval_print(args: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let key = native_car(args)?;
    let data = match key {
        RispExp::Symbol(_) => {
            env.get(&key.to_string()).ok_or(RispErr::Reason(format!("unexpected symbol='{}'",key)))?
        },
        _ => key,
    };
    Ok(data)
}

fn eval_print_env(env: &RispEnv) -> RispResult {
    for (k ,v) in env.data.iter() {
        println!("{}:{}", k, v);
    }
    Ok(RispExp::Nil)
}

pub fn native_car(list: &[RispExp]) -> RispResult {
    Ok(list.first()
        .ok_or(RispErr::Reason("expected a non empty list".to_string()))?.clone())
}

pub fn native_cdr(list: &[RispExp]) -> RispResult {
    let cdr = list[1..].to_vec();
    if cdr.len() == 0 {
        Ok(RispExp::Nil)
    } else {
        Ok(RispExp::List(cdr))
    }
}

fn eval_load_risp_file(args: &[RispExp], env: &mut RispEnv) -> RispResult {
    let path = args.first().ok_or(RispErr::Reason("expected first args".to_string()))?;
    let script = fs::read_to_string(path.to_string()).ok().ok_or(RispErr::Reason("can't load file".to_string()))?;
    Ok(parse_eval(script, env)?)
}

fn eval_lambda_arg(args: &[RispExp]) -> RispResult {
    let params = args.first().ok_or(RispErr::Reason("expected first args".to_string()))?;
    let body = args[1..].to_vec();

    Ok(RispExp::Lambda(RispLambda{
        params_exp: Rc::new(params.clone()),
        body_exp: Rc::new(body.clone()),
    }))
}

fn eval_if_arg(args: &[RispExp], env: &mut RispEnv) -> RispResult {
    let arg = args.first().ok_or(RispErr::Reason("expected first arg to be a bool".to_string()))?;
    let if_eval = eval(arg, env)?;
    match if_eval {
        RispExp::Bool(b) => {
            let idx = if b { 1 } else { 2 }; 
            let result = args.get(idx)
                .ok_or(RispErr::Reason(format!("expected args idx={}", idx)))?;

            eval(result, env)
        },
        _ => Err(RispErr::Reason(format!("unexpected if args = '{}'",arg)))
    }
}

fn eval_let_arg(args: &[RispExp], env: &mut RispEnv) -> RispResult {
    let [symbol_exp, value_exp] = <&[RispExp; 2]>::try_from(args).ok().ok_or(
        RispErr::Reason("Wrong number of arguments: let, 2".to_string())
    )?;
    let symbol = match symbol_exp {
        RispExp::Symbol(s) => Ok(s.clone()),
        _ => Err(RispErr::Reason("expected first arg to be a symbol".to_string()))
    }?;
    let value = eval(value_exp, env)?;
    env.data.insert(symbol, value);
    Ok(symbol_exp.clone())
}

fn eval_list(args: &[RispExp], env: &mut RispEnv) -> Result<Vec<RispExp>, RispErr> {
    args.iter().map(|x| eval(x, env)).collect()
}


fn env_for_lambda<'a>(params_exp: Rc<RispExp>, args: &[RispExp], outer_env: &'a mut RispEnv)
                      -> Result<RispEnv<'a>, RispErr> {
    let symbols = parse_list_of_symbol_strings(params_exp)?;
    if symbols.len() != args.len() {
        return Err(RispErr::Reason(format!("expected {} arguments, got {}", symbols.len(), args.len())))
    }
    let values = eval_list(args, outer_env)?;
    let mut data: HashMap<String, RispExp> = HashMap::new();
    for (k ,v) in symbols.iter().zip(values.iter()) {
        data.insert(k.clone(), v.clone());
    }
    Ok(RispEnv { data, outer: Some(outer_env) })
}

pub fn eval(exp: &RispExp, env: &mut RispEnv) -> RispResult {
    match exp {
        RispExp::Nil        => Ok(exp.clone()),
        RispExp::Bool(_)    => Ok(exp.clone()),
        RispExp::Number(_)  => Ok(exp.clone()),
        RispExp::Literal(_) => Ok(exp.clone()),
        RispExp::Symbol(k) => {
            env.get(k)
               .ok_or(RispErr::Reason(format!("unexpected symbol='{}'",k)))
               .map(|x| x.clone())
        },
        RispExp::List(list) => {
            let first = list
                .first()
                .ok_or(RispErr::Reason("expected a non empty list".to_string()))?;
            let args = &list[1..];
            match eval_built_in_func(first, args, env) {
                Some(res) => res,
                None => {
                    let first_eval = eval(first, env)?;
                    match first_eval {
                        RispExp::Func(f) => {
                            f(&eval_list(args, env)?)
                        },
                        RispExp::Lambda(lambda) => {
                            let local_env = &mut env_for_lambda(lambda.params_exp, args, env)?;
                            Ok(eval_list(lambda.body_exp.as_ref(), local_env)?.last().unwrap().clone())
                        }
                        _ => Err(RispErr::Reason(format!("Invalid function: {}", first)))
                    }
                }
            }
        },
        RispExp::Func(_) => Err(RispErr::Reason("unexpected syntax".to_string())),
        RispExp::Lambda(_) => Err(RispErr::Reason("unexpected syntax".to_string())),
    }
}

