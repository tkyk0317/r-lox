use crate::ast::AstType;
use crate::environment::{Environment, Value};
use std::error;
use std::fmt;

/// ランタイムエラー
pub enum RuntimeError {
    OperandType(Operand),
    TwoOperandType(Operand, Operand),
    NotFoundVar(String),
    NotFoundFunc(String),
    NotMatchArgsNum,
}
impl RuntimeError {
    fn operand_type(&self, operand: &Operand) -> Option<&str> {
        if one_type_check_string(operand) {
            Some("String")
        } else if one_type_check_f64(operand) {
            Some("f64")
        } else if one_type_check_bool(operand) {
            Some("bool")
        } else {
            None
        }
    }

    fn print(&self) -> String {
        match self {
            Self::OperandType(o) => format!("invalid type: {:?}", self.operand_type(o)),
            Self::TwoOperandType(l, r) => format!(
                "invalid type: left={:?} right={:?}",
                self.operand_type(l),
                self.operand_type(r)
            ),
            Self::NotFoundVar(v) => format!("Could not found variable: {:?}", v),
            Self::NotFoundFunc(v) => format!("Could not found function: {:?}", v),
            Self::NotMatchArgsNum => "Could not match Argument Length".to_string(),
        }
    }
}

impl fmt::Debug for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.print())
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.print())
    }
}

impl error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

// 評価結果
#[derive(Clone, Debug, PartialEq)]
pub enum ReturnType {
    Bool(bool),
    F64(f64),
    Void,
    String(String),
    Return(Box<ReturnType>),
}
pub type Operand = ReturnType;
type EvalResult = Result<Operand, RuntimeError>;

/// AST評価
///
/// # Arguments
/// * `ast` - AST
pub fn eval(ast: &AstType, env: &mut Environment) -> EvalResult {
    match ast {
        AstType::True => Ok(ReturnType::Bool(true)),
        AstType::False => Ok(ReturnType::Bool(false)),
        AstType::Nil => Ok(ReturnType::Void),
        AstType::Number(n) => Ok(ReturnType::F64(*n)),
        AstType::String(s) => Ok(ReturnType::String(s.clone())),
        AstType::Bang(o) => bang(eval(o, env)?),
        AstType::UnaryMinus(o) => unary_minus(eval(o, env)?),
        AstType::Plus(l, r) => plus(eval(l, env)?, eval(r, env)?),
        AstType::Minus(l, r) => minus(eval(l, env)?, eval(r, env)?),
        AstType::Mul(l, r) => mul(eval(l, env)?, eval(r, env)?),
        AstType::Div(l, r) => div(eval(l, env)?, eval(r, env)?),
        AstType::EqualEqual(l, r) => equal_equal(eval(l, env)?, eval(r, env)?),
        AstType::BangEqual(l, r) => bang_equal(eval(l, env)?, eval(r, env)?),
        AstType::Greater(l, r) => greater(eval(l, env)?, eval(r, env)?),
        AstType::Less(l, r) => less(eval(l, env)?, eval(r, env)?),
        AstType::GreaterEqual(l, r) => greater_equal(eval(l, env)?, eval(r, env)?),
        AstType::LessEqual(l, r) => less_equal(eval(l, env)?, eval(r, env)?),
        AstType::Print(o) => print_stmt(eval(o, env)?),
        AstType::Var(i, o) => var_decl(i, o, env),
        AstType::Identifier(i) => identifier(i, env),
        AstType::Assign(i, o) => assign(i, eval(o, env)?, env),
        AstType::Grouping(o) => eval(o, env),
        AstType::Block(o) => block(o, env),
        AstType::If(cond, if_stmt, else_stmt) => if_eval(cond, if_stmt, else_stmt, env),
        AstType::Or(left, right) => or_eval(eval(left, env)?, eval(right, env)?),
        AstType::And(left, right) => and_eval(eval(left, env)?, eval(right, env)?),
        AstType::While(cond, stmt) => while_eval(cond, stmt, env),
        AstType::Call(callee, arguments) => call_eval(callee, arguments, env),
        AstType::Fun(fun_name, arguments, block) => fun_eval(fun_name, arguments, block, env),
        AstType::Return(o) => return_eval(o, env),
    }
}

/// 評価結果出力
pub fn print(result: Operand) {
    if one_type_check_f64(&result) {
        println!("{}", downcast_f64(result))
    } else if one_type_check_string(&result) {
        println!("{}", downcast_string(result))
    } else if one_type_check_bool(&result) {
        println!("{}", downcast_bool(result))
    }
}

/// プラス演算子評価
///
/// # Arguments
// * `right` - 右オペランド
///
/// # Return
/// * Operand - 評価後の値（f64 or String）
fn plus(left: Operand, right: Operand) -> EvalResult {
    if type_check_f64(&left, &right) {
        Ok(ReturnType::F64(downcast_f64(left) + downcast_f64(right)))
    } else if type_check_string(&left, &right) {
        Ok(ReturnType::String(format!(
            "{}{}",
            downcast_string(left),
            downcast_string(right)
        )))
    } else {
        Err(RuntimeError::TwoOperandType(left, right))
    }
}

/// マイナス演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * EvalResult - 評価後の値（f64）
fn minus(left: Operand, right: Operand) -> EvalResult {
    if type_check_f64(&left, &right) {
        Ok(ReturnType::F64(downcast_f64(left) - downcast_f64(right)))
    } else {
        Err(RuntimeError::TwoOperandType(left, right))
    }
}

/// 積算演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * EvalResult - 評価後の値（f64）
fn mul(left: Operand, right: Operand) -> EvalResult {
    if type_check_f64(&left, &right) {
        Ok(ReturnType::F64(downcast_f64(left) * downcast_f64(right)))
    } else {
        Err(RuntimeError::TwoOperandType(left, right))
    }
}

/// 除算演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * EvalResult - 評価後の値（f64）
fn div(left: Operand, right: Operand) -> EvalResult {
    if type_check_f64(&left, &right) {
        Ok(ReturnType::F64(downcast_f64(left) / downcast_f64(right)))
    } else {
        Err(RuntimeError::TwoOperandType(left, right))
    }
}

/// -演算子評価
///
/// # Arguments
/// * `operand` - オペランド
///
/// # Return
/// * EvalResult - 評価後の値（f64）
fn unary_minus(operand: Operand) -> EvalResult {
    if one_type_check_f64(&operand) {
        Ok(ReturnType::F64(-(downcast_f64(operand))))
    } else {
        Err(RuntimeError::OperandType(operand))
    }
}

/// !演算子評価
///
/// # Arguments
/// * `operand` - オペランド
///
/// # Return
/// * EvalResult - 評価後の値（bool）
fn bang(operand: Operand) -> EvalResult {
    if one_type_check_bool(&operand) {
        Ok(ReturnType::Bool(!downcast_bool(operand)))
    } else if one_type_check_void(&operand) {
        Ok(ReturnType::Bool(true))
    } else {
        Ok(ReturnType::Bool(false))
    }
}

/// ==演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * EvalResult - 評価後の値（bool）
fn equal_equal(left: Operand, right: Operand) -> EvalResult {
    if type_check_f64(&left, &right) {
        Ok(ReturnType::Bool(downcast_f64(left) == downcast_f64(right)))
    } else if type_check_string(&left, &right) {
        Ok(ReturnType::Bool(
            downcast_string(left) == downcast_string(right),
        ))
    } else if type_check_bool(&left, &right) {
        Ok(ReturnType::Bool(
            downcast_bool(left) == downcast_bool(right),
        ))
    } else {
        Err(RuntimeError::TwoOperandType(left, right))
    }
}

/// !=演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * EvalResult - 評価後の値（bool）
fn bang_equal(left: Operand, right: Operand) -> EvalResult {
    let ret = equal_equal(left, right)?;

    match ret {
        ReturnType::Bool(ret) => Ok(ReturnType::Bool(!ret)),
        _ => Err(RuntimeError::OperandType(ret)),
    }
}

/// >演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * EvalResult - 評価後の値（bool）
fn greater(left: Operand, right: Operand) -> EvalResult {
    if type_check_f64(&left, &right) {
        Ok(ReturnType::Bool(downcast_f64(left) > downcast_f64(right)))
    } else if type_check_string(&left, &right) {
        Ok(ReturnType::Bool(
            downcast_string(left) > downcast_string(right),
        ))
    } else if type_check_bool(&left, &right) {
        Ok(ReturnType::Bool(
            downcast_bool(left) & !(downcast_bool(right)),
        ))
    } else {
        Err(RuntimeError::TwoOperandType(left, right))
    }
}

/// >=演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * EvalResult - 評価後の値（bool）
fn greater_equal(left: Operand, right: Operand) -> EvalResult {
    if type_check_f64(&left, &right) {
        Ok(ReturnType::Bool(downcast_f64(left) >= downcast_f64(right)))
    } else if type_check_string(&left, &right) {
        Ok(ReturnType::Bool(
            downcast_string(left) >= downcast_string(right),
        ))
    } else if type_check_bool(&left, &right) {
        Ok(ReturnType::Bool(
            downcast_bool(left) >= downcast_bool(right),
        ))
    } else {
        Err(RuntimeError::TwoOperandType(left, right))
    }
}

/// <演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * EvalResult - 評価後の値（bool）
fn less(left: Operand, right: Operand) -> EvalResult {
    if type_check_f64(&left, &right) {
        Ok(ReturnType::Bool(downcast_f64(left) < downcast_f64(right)))
    } else if type_check_string(&left, &right) {
        Ok(ReturnType::Bool(
            downcast_string(left) < downcast_string(right),
        ))
    } else if type_check_bool(&left, &right) {
        Ok(ReturnType::Bool(
            !downcast_bool(left) & downcast_bool(right),
        ))
    } else {
        Err(RuntimeError::TwoOperandType(left, right))
    }
}

/// <=演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * EvalResult - 評価後の値（bool）
fn less_equal(left: Operand, right: Operand) -> EvalResult {
    if type_check_f64(&left, &right) {
        Ok(ReturnType::Bool(downcast_f64(left) <= downcast_f64(right)))
    } else if type_check_string(&left, &right) {
        Ok(ReturnType::Bool(
            downcast_string(left) <= downcast_string(right),
        ))
    } else if type_check_bool(&left, &right) {
        Ok(ReturnType::Bool(
            downcast_bool(left) <= downcast_bool(right),
        ))
    } else {
        Err(RuntimeError::TwoOperandType(left, right))
    }
}

/// print文評価
///
/// # Arguments
/// * `operand` - オペランド
///
/// # Return
/// * EvalResult - 評価後の値（f64）
fn print_stmt(operand: Operand) -> EvalResult {
    print(operand);

    Ok(ReturnType::Void)
}

/// 変数定義評価
///
/// # Arguments
/// * `i` - 変数名
/// * `right` - 初期化式
///
/// # Return
/// * EvalResult - 評価後の値
fn var_decl(i: &String, operand: &AstType, env: &mut Environment) -> EvalResult {
    let right = eval(operand, env)?;
    let value = to_env_value(right);
    env.define(i.to_string(), value);

    Ok(ReturnType::Void)
}

/// 変数参照評価
///
/// # Arguments
/// * `i` - 変数名
///
/// # Return
/// * EvalResult - 評価後の値（bool/f64/String）
fn identifier(i: &String, env: &mut Environment) -> EvalResult {
    let val = env.get(i);
    if let Some(val) = val {
        match val {
            Value::F64(f) => Ok(ReturnType::F64(*f)),
            Value::String(s) => Ok(ReturnType::String(s.to_string())),
            Value::Bool(b) => Ok(ReturnType::Bool(*b)),
            _ => Err(RuntimeError::NotFoundVar(i.to_string())),
        }
    } else {
        Err(RuntimeError::NotFoundVar(i.to_string()))
    }
}

/// 代入式評価
///
/// # Arguments
/// * `i` - 変数名
/// * `right` - 初期化式
///
/// # Return
/// * EvalResult - 評価後の値
fn assign(i: &String, right: Operand, env: &mut Environment) -> EvalResult {
    let val = env.get(i);
    if val.is_some() {
        // 変数に対する値を更新
        let value = to_env_value(right);
        env.push(i.to_string(), value);

        Ok(ReturnType::Void)
    } else {
        Err(RuntimeError::NotFoundVar(i.to_string()))
    }
}

/// 環境に格納する値を取得
/// # Arguments
/// * `operand` - オペランド
///
/// # Return
/// * Value - 変換後のValue
fn to_env_value(operand: Operand) -> Value {
    if one_type_check_string(&operand) {
        Value::String(downcast_string(operand))
    } else if one_type_check_f64(&operand) {
        Value::F64(downcast_f64(operand))
    } else {
        Value::Bool(downcast_bool(operand))
    }
}

/// Block評価
///
/// # Arguments
/// * `o` - Ast配列
///
/// # Return
/// * EvalResult - 評価後の値
fn block(ast_arr: &Vec<AstType>, env: &mut Environment) -> EvalResult {
    // ブロック内の環境を作成
    let mut ret: EvalResult = Ok(ReturnType::Void);
    let mut block_env = Environment::with_enclosing(env.clone());

    for ast in ast_arr {
        ret = eval(ast, &mut block_env);
        match ret {
            Ok(ReturnType::Return(_)) => break,
            _ => continue,
        }
    }

    // ブロック内で更新された環境で上書き
    *env = *block_env.enclosing.unwrap().clone();

    ret
}

/// if文評価
///
/// # Arguments
/// * `cond` - 条件式
/// * `if_stmt` - ifブロック
/// * `else_stmt` - elseブロック
///
/// # Return
/// * EvalResult - 評価後の値
fn if_eval(
    cond: &AstType,
    if_stmt: &AstType,
    else_stmt: &AstType,
    env: &mut Environment,
) -> EvalResult {
    let result = downcast_bool(eval(cond, env)?);
    if result {
        Ok(eval(if_stmt, env)?)
    } else {
        Ok(eval(else_stmt, env)?)
    }
}

/// while文評価
///
/// # Arguments
/// * `cond` - 条件式
/// * `stmt` - ブロック
///
/// # Return
/// * EvalResult - 評価後の値
fn while_eval(cond: &AstType, stmt: &AstType, env: &mut Environment) -> EvalResult {
    loop {
        let cond_ret = downcast_bool(eval(cond, env)?);
        if !cond_ret {
            break;
        }
        eval(stmt, env)?;
    }

    Ok(ReturnType::Void)
}

/// call評価
///
/// # Arguments
/// * `callee` - 関数名などのcallee
/// * `arguments` - 引数列
///
/// # Return
/// * EvalResult - 評価後の値
fn call_eval(callee: &String, arguments: &[AstType], env: &mut Environment) -> EvalResult {
    let args_val: Vec<_> = arguments
        .iter()
        .map(|arg| eval(arg, env))
        .map(Result::unwrap)
        .collect();

    if let Some(func) = env.clone().get(callee) {
        match func {
            Value::UserFunc(args, body) => call_func(body, args, &args_val, env),
            Value::EmbeddedFunc(f) => {
                f();

                Ok(ReturnType::Void)
            }
            _ => Err(RuntimeError::NotFoundFunc(callee.to_string())),
        }
    } else {
        Err(RuntimeError::NotFoundFunc(callee.to_string()))
    }
}

/// call function
///
/// # Arguments
/// * `func` - 関数内容
/// * `args` - 引数列定義
/// * `args_val` - 引数値
/// * `env` - 環境
///
/// # Return
/// * EvalResult - 評価後の値
fn call_func(
    body: &AstType,
    args: &[AstType],
    args_val: &Vec<Operand>,
    env: &mut Environment,
) -> EvalResult {
    if args.len() != args_val.len() {
        return Err(RuntimeError::NotMatchArgsNum);
    }

    // 引数の内容を環境に設定
    let mut block_env = Environment::with_enclosing(env.clone());
    args.iter().zip(args_val).for_each(|(var_name, value)| {
        if let AstType::Identifier(key) = var_name {
            let arg = if one_type_check_string(value) {
                Value::String(downcast_string(value.clone()))
            } else if one_type_check_f64(value) {
                Value::F64(downcast_f64(value.clone()))
            } else {
                Value::Bool(downcast_bool(value.clone()))
            };
            block_env.define(key.to_string(), arg);
        }
    });

    // 関数評価
    let result = eval(body, &mut block_env)?;

    // ブロック内で更新された環境で上書き
    *env = *block_env.enclosing.unwrap().clone();

    Ok(result)
}

/// return評価
///
/// # Arguments
/// * `operand` - オペランド
///
/// # Return
/// * EvalResult - 評価後の値
fn return_eval(operand: &AstType, env: &mut Environment) -> EvalResult {
    let ret = eval(operand, env)?;

    Ok(ReturnType::Return(Box::new(ret)))
}

/// fun評価
///
/// # Arguments
/// * `fun_name` - 関数名
/// * `arguments` - 引数列
/// * `block` - ブロック
/// * `env` - 環境
///
/// # Return
/// * EvalResult - 評価後の値
fn fun_eval(
    fun_name: &String,
    arguments: &[AstType],
    block: &AstType,
    env: &mut Environment,
) -> EvalResult {
    // 関数定義を環境へ追加
    env.define(
        fun_name.to_string(),
        Value::UserFunc(arguments.to_owned(), block.clone()),
    );

    Ok(ReturnType::Void)
}

/// or評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * EvalResult - 評価後の値
fn or_eval(left: Operand, right: Operand) -> EvalResult {
    if type_check_bool(&left, &right) {
        let (l, r) = (downcast_bool(left), downcast_bool(right));
        Ok(ReturnType::Bool(l || r))
    } else {
        Err(RuntimeError::TwoOperandType(left, right))
    }
}

/// and評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * EvalResult - 評価後の値
fn and_eval(left: Operand, right: Operand) -> EvalResult {
    if type_check_bool(&left, &right) {
        let (l, r) = (downcast_bool(left), downcast_bool(right));
        Ok(ReturnType::Bool(l && r))
    } else {
        Err(RuntimeError::TwoOperandType(left, right))
    }
}

/// オペランド型チェック
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * bool
fn type_check_string(left: &Operand, right: &Operand) -> bool {
    one_type_check_string(left) && one_type_check_string(right)
}
fn type_check_f64(left: &Operand, right: &Operand) -> bool {
    one_type_check_f64(left) && one_type_check_f64(right)
}
fn type_check_bool(left: &Operand, right: &Operand) -> bool {
    one_type_check_bool(left) && one_type_check_bool(right)
}

/// オペランド型チェック
///
/// # Arguments
/// * `operand` - オペランド
///
/// # Return
/// * bool
fn one_type_check_string(operand: &Operand) -> bool {
    match operand {
        ReturnType::Return(o) => one_type_check_string(o),
        _ => matches!(*operand, ReturnType::String(_)),
    }
}
fn one_type_check_f64(operand: &Operand) -> bool {
    match operand {
        ReturnType::Return(o) => one_type_check_f64(o),
        _ => matches!(*operand, ReturnType::F64(_)),
    }
}
fn one_type_check_bool(operand: &Operand) -> bool {
    match operand {
        ReturnType::Return(o) => one_type_check_bool(o),
        _ => matches!(*operand, ReturnType::Bool(_)),
    }
}
fn one_type_check_void(operand: &Operand) -> bool {
    match operand {
        ReturnType::Return(o) => one_type_check_void(o),
        _ => matches!(*operand, ReturnType::Void),
    }
}

/// ダウンキャスト
///
/// # Arguments
/// * `operand` - オペランド
///
/// # Return
/// * f64/String/book - ダウンキャスト後のvalue
fn downcast_string(operand: Operand) -> String {
    match operand {
        ReturnType::String(s) => s.clone(),
        ReturnType::Return(s) => match *s {
            ReturnType::String(s) => s.clone(),
            _ => panic!("[downcast_string] support only String"),
        },
        _ => panic!("[downcast_string] support only String"),
    }
}
fn downcast_f64(operand: Operand) -> f64 {
    match operand {
        ReturnType::F64(f) => f,
        ReturnType::Return(f) => match *f {
            ReturnType::F64(f) => f,
            _ => panic!("[downcast_f64] support only f64"),
        },
        _ => panic!("[downcast_f64] support only f64"),
    }
}
fn downcast_bool(operand: Operand) -> bool {
    match operand {
        ReturnType::Bool(b) => b,
        ReturnType::Return(b) => match *b {
            ReturnType::Bool(b) => b,
            _ => panic!("[downcast_bool] support only bool"),
        },
        _ => panic!("[downcast_bool] support only bool"),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn リテラル_eval() {
        let ast = AstType::Number(1.0);
        let mut env = Environment::new();
        assert_eq!(1.0, downcast_f64(eval(&ast, &mut env).unwrap()));

        let ast = AstType::String("test".to_string());
        let mut env = Environment::new();
        assert_eq!("test", downcast_string(eval(&ast, &mut env).unwrap()));

        let ast = AstType::True;
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::False;
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn 加算_eval() {
        let ast = AstType::Plus(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert_eq!(3.0, downcast_f64(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Plus(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Plus(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(3.0)),
            )),
        );
        let mut env = Environment::new();
        assert_eq!(6.0, downcast_f64(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn 減算_eval() {
        let ast = AstType::Minus(
            Box::new(AstType::Number(3.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert_eq!(1.0, downcast_f64(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Minus(
            Box::new(AstType::Minus(
                Box::new(AstType::Number(10.0)),
                Box::new(AstType::Number(3.0)),
            )),
            Box::new(AstType::Number(1.0)),
        );
        let mut env = Environment::new();
        assert_eq!(6.0, downcast_f64(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn 文字列連結_eval() {
        let ast = AstType::Plus(
            Box::new(AstType::String(String::from("test,"))),
            Box::new(AstType::String(String::from("hello"))),
        );
        let mut env = Environment::new();
        assert_eq!("test,hello", downcast_string(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn 積算_eval() {
        let ast = AstType::Mul(
            Box::new(AstType::Number(3.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert_eq!(6.0, downcast_f64(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Mul(
            Box::new(AstType::Mul(
                Box::new(AstType::Number(10.0)),
                Box::new(AstType::Number(3.0)),
            )),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert_eq!(60.0, downcast_f64(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn 除算_eval() {
        let ast = AstType::Div(
            Box::new(AstType::Number(6.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert_eq!(3.0, downcast_f64(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Div(
            Box::new(AstType::Div(
                Box::new(AstType::Number(30.0)),
                Box::new(AstType::Number(3.0)),
            )),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert_eq!(5.0, downcast_f64(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn unary_minus_eval() {
        let ast = AstType::UnaryMinus(Box::new(AstType::Number(1.0)));
        let mut env = Environment::new();
        assert_eq!(-1.0, downcast_f64(eval(&ast, &mut env).unwrap()));

        let ast = AstType::UnaryMinus(Box::new(AstType::Plus(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(4.0)),
        )));
        let mut env = Environment::new();
        assert_eq!(-5.0, downcast_f64(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn unary_bang_eval() {
        let ast = AstType::Bang(Box::new(AstType::Number(1.0)));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Bang(Box::new(AstType::Nil));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Bang(Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Bang(Box::new(AstType::False));
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Bang(Box::new(AstType::String(String::from("a"))));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn equal_equal_eval() {
        let ast = AstType::EqualEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(1.0)),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::EqualEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::EqualEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test"))),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::EqualEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test, test"))),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::EqualEqual(Box::new(AstType::True), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::EqualEqual(Box::new(AstType::False), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::EqualEqual(Box::new(AstType::False), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn bang_equal_eval() {
        let ast = AstType::BangEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(1.0)),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::BangEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::BangEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test"))),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::BangEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test, test"))),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::BangEqual(Box::new(AstType::True), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::BangEqual(Box::new(AstType::False), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::BangEqual(Box::new(AstType::False), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn greater_eval() {
        let ast = AstType::Greater(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Greater(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Greater(Box::new(AstType::True), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Greater(Box::new(AstType::False), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Greater(Box::new(AstType::False), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Greater(Box::new(AstType::True), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn less_eval() {
        let ast = AstType::Less(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Less(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Less(Box::new(AstType::True), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Less(Box::new(AstType::False), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Less(Box::new(AstType::False), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Less(Box::new(AstType::True), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn greater_equal_eval() {
        let ast = AstType::GreaterEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::GreaterEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::GreaterEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("a"))),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::GreaterEqual(Box::new(AstType::True), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::GreaterEqual(Box::new(AstType::False), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::GreaterEqual(Box::new(AstType::False), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::GreaterEqual(Box::new(AstType::True), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn less_equal_eval() {
        let ast = AstType::LessEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::LessEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::LessEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("a"))),
        );
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::LessEqual(Box::new(AstType::True), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::LessEqual(Box::new(AstType::False), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::LessEqual(Box::new(AstType::False), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::LessEqual(Box::new(AstType::True), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn if_eval() {
        let ast = AstType::If(
            Box::new(AstType::Less(
                Box::new(AstType::Number(1.0)),
                Box::new(AstType::Number(2.0)),
            )),
            Box::new(AstType::Number(3.0)),
            Box::new(AstType::Number(4.0)),
        );
        let mut env = Environment::new();
        assert_eq!(3.0, downcast_f64(eval(&ast, &mut env).unwrap()));

        let ast = AstType::If(
            Box::new(AstType::Greater(
                Box::new(AstType::Number(1.0)),
                Box::new(AstType::Number(2.0)),
            )),
            Box::new(AstType::Number(3.0)),
            Box::new(AstType::Number(4.0)),
        );
        let mut env = Environment::new();
        assert_eq!(4.0, downcast_f64(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn or_eval() {
        let ast = AstType::Or(Box::new(AstType::True), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Or(Box::new(AstType::True), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::Or(Box::new(AstType::False), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn and_eval() {
        let ast = AstType::And(Box::new(AstType::True), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::And(Box::new(AstType::True), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));

        let ast = AstType::And(Box::new(AstType::False), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(!downcast_bool(eval(&ast, &mut env).unwrap()));
    }

    #[test]
    fn call_eval() {
        let ast = AstType::Call("clock".to_string(), vec![]);
        let mut env = Environment::new();
        env = crate::embedded::func::register_func(&env);

        assert_eq!(ReturnType::Void, eval(&ast, &mut env).unwrap());
    }
}
