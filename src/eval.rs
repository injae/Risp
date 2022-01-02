use crate::risp_type::*;
use crate::parser::*;
use std::collections::HashMap;
use std::rc::Rc;

fn eval_built_in_func(exp: &RispExp, args: &[RispExp], env: &mut RispEnv) -> Option<RispResult> {
    match exp {
        RispExp::Symbol(s) => {
            match s.as_ref() {
                "if"  => Some(eval_if_arg(args, env)),
                "let" => Some(eval_let_arg(args, env)),
                "fn" => Some(eval_lamgda_arg(args)),
                _ => None,
            }
        }
        _ => None
    }
}

fn eval_lamgda_arg(args: &[RispExp]) -> Result<RispExp, RispErr> {
    let params = args.first().ok_or(RispErr::Reason("expected args args".to_string()))?;
    let body = args.get(1).ok_or(RispErr::Reason("expected second args".to_string()))?;

    if args.len() > 2 {
        return Err(RispErr::Reason("lambda definition can only have two args".to_string()))
    }

    Ok(RispExp::Lambda(RispLambda{
        body_exp: Rc::new(body.clone()),
        params_exp: Rc::new(params.clone()),
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
    let first_arg = args.first().ok_or(RispErr::Reason("expected first argument".to_string()))?;
    let symbol = match first_arg {
        RispExp::Symbol(s) => Ok(s.clone()),
        _ => Err(RispErr::Reason("expected first arg to be a symbol".to_string()))
    }?;
    let value = args.get(1).ok_or(RispErr::Reason("expected second arg".to_string()))?;
    if args.len() > 2 {
        return Err(RispErr::Reason("let can only have two args".to_string()));
    }
    let eval_value = eval(value, env)?;
    env.data.insert(symbol, eval_value);
    Ok(first_arg.clone())
}

fn eval_list(args: &[RispExp], env: &mut RispEnv) -> Result<Vec<RispExp>, RispErr> {
    args.iter().map(|x| eval(x, env)).collect()
}


fn env_for_lambda<'a>(params_exp: Rc<RispExp>, args: &[RispExp], outer_env: &'a mut RispEnv) -> Result<RispEnv<'a>, RispErr> {
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
        RispExp::Symbol(k) => {
            env.get(k)
               .ok_or(RispErr::Reason(format!("unexpected symbol='{}'",k)))
               .map(|x| x.clone())
        },
        RispExp::Nil => Ok(exp.clone()),
        RispExp::Bool(_)   => Ok(exp.clone()),
        RispExp::Number(_) => Ok(exp.clone()),
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
                            eval(&lambda.body_exp, &mut env_for_lambda(lambda.params_exp, args, env)?)
                        }
                        _ => Ok(exp.clone())
                    }
                }
            }
        },
        RispExp::Func(_) => Err(RispErr::Reason("unexpected syntax".to_string())),
        RispExp::Lambda(_) => Err(RispErr::Reason("unexpected syntax".to_string())),
    }
}

