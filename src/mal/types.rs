use alloc::rc::Rc;
use alloc::string::{String,ToString};
use crate::mal::types::MalErr::ErrString;
use alloc::vec::Vec;
use core::cell::RefCell;
use hashbrown::HashMap;

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
        // eval: fn(ast: MalVal, env: Env) -> MalRet,
        ast: Rc<MalVal>, // 函数 抽象语法树
        // env: Env,    // repl 环境
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

// type utility functions
  
//抛出错误
pub fn error(s: &str) -> MalRet {
    Err(ErrString(s.to_string()))
}