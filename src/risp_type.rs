use anyhow::Result;
use core::fmt;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;
use thiserror::Error;

#[derive(Clone)]
pub enum RispExp {
    Nil,
    Bool(bool),
    Symbol(String),
    Literal(String),
    Number(f64),
    List(Vec<RispExp>),
    Func(fn(&[RispExp]) -> Result<RispExp, RispErr>),
    Lambda(RispLambda),
}

#[derive(Error, Debug)]
pub enum RispErr {
    #[error("`{0}`")]
    Reason(String),
    #[error("Unexpected Symbol='{0}'")]
    UnexpectedSymbol(String),
    #[error("Unexpected Syntax")]
    UnexpectedSyntax,
    #[error("Invalid Function: '{0}'")]
    InvalidFunction(String),
    #[error("Invalid Arguments")]
    InvalidArgs,
    #[error("IO Error `{0}`")]
    IOError(#[from] std::io::Error),
    #[error("unknown data store error")]
    Unknown,
}

//impl Display for RispErr {
//    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//        let str = match &self {
//            RispErr::Reason(s) => s,
//            RispErr::IOError(e) => &e.to_string(),
//        };
//        write!(f, "{str}")
//    }
//}

#[derive(Clone)]
pub struct RispEnv<'a> {
    pub data: HashMap<String, RispExp>,
    pub outer: Option<&'a RispEnv<'a>>,
}

impl<'a> RispEnv<'a> {
    pub fn get(&self, key: &str) -> Option<RispExp> {
        match self.data.get(key) {
            Some(exp) => Some(exp.clone()),
            None => match &self.outer {
                Some(outer_env) => outer_env.get(key),
                None => None,
            },
        }
    }
}

#[derive(Clone)]
pub struct RispLambda {
    pub params_exp: Rc<RispExp>,
    pub body_exp: Rc<Vec<RispExp>>,
}

pub type RispResult = Result<RispExp, RispErr>;

impl fmt::Display for RispExp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            RispExp::Nil => "nil".to_string(),
            RispExp::Bool(b) => b.to_string(),
            RispExp::Number(n) => n.to_string(),
            RispExp::Symbol(s) => s.clone(),
            RispExp::List(list) => {
                let xs: Vec<String> = list.iter().map(|x| x.to_string()).collect();
                format!("({})", xs.join(","))
            }
            RispExp::Func(_) => "Function".to_string(),
            RispExp::Lambda(_) => "Lambda".to_string(),
            RispExp::Literal(s) => s.clone(),
        };

        write!(f, "{str}")
    }
}
