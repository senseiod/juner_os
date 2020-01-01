use alloc::rc::Rc;
use alloc::string::{String,ToString};
use crate::mal::types::MalErr::{ErrString,ErrMalVal};
use alloc::vec::Vec;
use core::cell::RefCell;
use hashbrown::HashMap;
use crate::mal::types::MalVal::{Hash,Str,Nil,Func,Bool,Int,Sym,List,Vector,MalFunc,Atom};
use core::fmt;
use crate::mal::env::{Env,env_bind};

#[derive(Debug,Clone)]
pub enum MalVal{
    Nil,
    Bool(bool), //布尔类型
    Int(i64),   // int类型
    Str(String), // 字符串类型
    Sym(String), 
    List(Rc<Vec<MalVal>>, Rc<MalVal>),  // 列表类型
    Vector(Rc<Vec<MalVal>>, Rc<MalVal>), // 向量类型
    Hash(Rc<HashMap<String,MalVal>>,Rc<MalVal>), // hashMap 类型
    Func(fn(MalArgs) -> MalRet,Rc<MalVal>), //函数 相当于 lamdba (x)-> M
    MalFunc {
        eval: fn(ast: MalVal, env: Env) -> MalRet,
        ast: Rc<MalVal>, // 函数 抽象语法树
        env: Env,    // repl 环境
        params: Rc<MalVal>,  // 参数值  TODO： 其实可以单值然后用柯里化
        is_macro: bool,    // 是否是宏
        meta: Rc<MalVal>,   // 元数据
    },
    Atom(Rc<RefCell<MalVal>>) //原子
}

// Mal 报错结构
#[derive(Debug)]
pub enum MalErr {
    ErrString(String),
    ErrMalVal(MalVal),
}

// Mal 入参
pub type MalArgs = Vec<MalVal>;
// Mal 出参结构
pub type MalRet = Result<MalVal, MalErr>;

#[macro_export]
macro_rules! list {
    ($seq:expr) => {{
      List(Rc::new($seq),Rc::new(Nil))
    }};
    [$($args:expr),*] => {{
      let v: Vec<MalVal> = vec![$($args),*];
      List(Rc::new(v),Rc::new(Nil))
    }}
}

#[macro_export]
macro_rules! vector {
    ($seq:expr) => {{
      Vector(Rc::new($seq),Rc::new(Nil))
    }};
    [$($args:expr),*] => {{
      let v: Vec<MalVal> = vec![$($args),*];
      Vector(Rc::new(v),Rc::new(Nil))
    }}
}

#[macro_export]
macro_rules! vec {
    ($elem:expr;$n:expr) => {
        $crate::alloc::vec::from_elem($elem, n)
    };
    ($($x:expr),*) => {
        <[_]>::into_vec(box[$($x),*])
    };
    ($($x:expr,)*) => {$crate::vec![$($x),*]}
}

#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => ($crate::alloc::fmt::format(format_args!($($arg)*)))
}

// type utility functions
  
//抛出错误
pub fn error(s: &str) -> MalRet {
    Err(ErrString(s.to_string()))
}

//格式化错误输出
pub fn format_error(e: MalErr) -> String {
    match e {
        ErrString(s) => s.clone(),
        ErrMalVal(mv) => mv.pr_str(true),
    }
}

// 把参数 变成hashmap
pub fn _assoc(mut hm: HashMap<String, MalVal>, kvs: MalArgs) -> MalRet {
    if kvs.len() % 2 != 0 {
        return error("odd number of elements");
    }
    let mut itre = kvs.iter();
    loop{
        let k = itre.next();
        match k {
            Some(Str(s))=>{
                match itre.next() {
                    Some(v) => {
                        hm.insert(s.to_string(), v.clone());
                    },
                    // 这里应该永远也不会发生
                    None => return error("key to value,vlaue is not a MalVal"),
                }
            },
            None => break,
            _ => return error("key is not string"),
        }
    }
    Ok(Hash(Rc::new(hm), Rc::new(Nil)))
}

// 创建hashmap
pub fn hash_map(kvs: MalArgs) -> MalRet {
    let hm: HashMap<String, MalVal> = HashMap::new();
    _assoc(hm, kvs)
}

// 创建一个函数
pub fn func(f: fn(MalArgs) -> MalRet) -> MalVal {
    Func(f, Rc::new(Nil))
}

// 实现比较方法 判断两个 MalVal 是否相等
impl PartialEq for MalVal {
    fn eq(&self, other: &MalVal) -> bool {
        match (self, other) {
            (Nil, Nil) => true,
            (Bool(ref a), Bool(ref b)) => a == b,
            (Int(ref a), Int(ref b)) => a == b,
            (Str(ref a), Str(ref b)) => a == b,
            (Sym(ref a), Sym(ref b)) => a == b,
            (List(ref a, _), List(ref b, _))
            | (Vector(ref a, _), Vector(ref b, _))
            | (List(ref a, _), Vector(ref b, _))
            | (Vector(ref a, _), List(ref b, _)) => a == b,
            (Hash(ref a, _), Hash(ref b, _)) => a == b,
            (MalFunc { .. }, MalFunc { .. }) => false, // 两个函数永远也不能相同！
            _ => false,
        }
    }
}

impl MalVal {

    pub fn apply(&self, args: MalArgs) -> MalRet {
        match *self {
            Func(f, _) => f(args),
            MalFunc {
                eval,
                ref ast,
                ref env,
                ref params,
                ..
            } => {
                let a = &**ast;
                let p = &**params;
                let fn_env = env_bind(Some(env.clone()), p.clone(), args)?;
                Ok(eval(a.clone(), fn_env)?)
            }
            _ => error("attempt to call non-function"),
        }
    }
    //todo
}
