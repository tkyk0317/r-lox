use crate::ast::AstType;
use crate::environment::{Environment, Value};
use std::any::{Any, TypeId};
use std::error;
use std::fmt;

/// ランタイムエラー
pub enum RuntimeError {
    OperandType(Operand),
    TwoOperandType(Operand, Operand),
    NotFoundVar(String),
}
impl RuntimeError {
    fn operand_type(&self, operand: &Operand) -> Option<&str> {
        if one_type_check::<String>(operand) {
            Some("String")
        } else if one_type_check::<f64>(operand) {
            Some("f64")
        } else if one_type_check::<bool>(operand) {
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

pub type Operand = Box<dyn Any>;
type EvalResult = Result<Operand, RuntimeError>;

/// AST評価
///
/// # Arguments
/// * `ast` - AST
pub fn eval(ast: &AstType, env: &mut Environment) -> EvalResult {
    match ast {
        AstType::True => Ok(Box::new(true)),
        AstType::False => Ok(Box::new(false)),
        AstType::Nil => Ok(Box::new(None::<()>)),
        AstType::Number(n) => Ok(Box::new(*n)),
        AstType::String(s) => Ok(Box::new(s.clone())),
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
        AstType::Var(i, o) => var_decl(i, eval(o, env)?, env),
        AstType::Identifier(i) => identifier(i, env),
        AstType::Assign(i, o) => assign(i, eval(o, env)?, env),
        AstType::Grouping(o) => eval(o, env),
        AstType::Block(o) => {
            // ブロック用の環境を作成
            let mut block_env = Environment::with_enclosing(env.clone());
            block(o, &mut block_env)
        }
        _ => panic!(""),
    }
}

/// 評価結果出力
pub fn print(result: Operand) {
    if one_type_check::<f64>(&result) {
        println!("{}", downcast::<f64>(result))
    } else if one_type_check::<String>(&result) {
        println!("{}", downcast::<String>(result))
    } else if one_type_check::<bool>(&result) {
        println!("{}", downcast::<bool>(result))
    }
}

/// プラス演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * Operand - 評価後の値（f64 or String）
fn plus(left: Operand, right: Operand) -> EvalResult {
    if type_check::<f64>(&left, &right) {
        Ok(Box::new(downcast::<f64>(left) + downcast::<f64>(right)))
    } else if type_check::<String>(&left, &right) {
        Ok(Box::new(format!(
            "{}{}",
            downcast::<String>(left),
            downcast::<String>(right)
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
    if type_check::<f64>(&left, &right) {
        Ok(Box::new(downcast::<f64>(left) - downcast::<f64>(right)))
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
    if type_check::<f64>(&left, &right) {
        Ok(Box::new(downcast::<f64>(left) * downcast::<f64>(right)))
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
    if type_check::<f64>(&left, &right) {
        Ok(Box::new(downcast::<f64>(left) / downcast::<f64>(right)))
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
    if one_type_check::<f64>(&operand) {
        Ok(Box::new(-(downcast::<f64>(operand))))
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
    if one_type_check::<bool>(&operand) {
        Ok(Box::new(!*operand.downcast::<bool>().unwrap()))
    } else if one_type_check::<Option<()>>(&operand) {
        Ok(Box::new(true))
    } else {
        Ok(Box::new(false))
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
    if type_check::<f64>(&left, &right) {
        Ok(Box::new(downcast::<f64>(left) == downcast::<f64>(right)))
    } else if type_check::<String>(&left, &right) {
        Ok(Box::new(
            downcast::<String>(left) == downcast::<String>(right),
        ))
    } else if type_check::<bool>(&left, &right) {
        Ok(Box::new(downcast::<bool>(left) == downcast::<bool>(right)))
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

    Ok(Box::new(!*ret.downcast::<bool>().unwrap()))
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
    if type_check::<f64>(&left, &right) {
        Ok(Box::new(downcast::<f64>(left) > downcast::<f64>(right)))
    } else if type_check::<String>(&left, &right) {
        Ok(Box::new(
            downcast::<String>(left) > downcast::<String>(right),
        ))
    } else if type_check::<bool>(&left, &right) {
        Ok(Box::new(
            downcast::<bool>(left) & !(downcast::<bool>(right)),
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
    if type_check::<f64>(&left, &right) {
        Ok(Box::new(downcast::<f64>(left) >= downcast::<f64>(right)))
    } else if type_check::<String>(&left, &right) {
        Ok(Box::new(
            downcast::<String>(left) >= downcast::<String>(right),
        ))
    } else if type_check::<bool>(&left, &right) {
        Ok(Box::new(downcast::<bool>(left) >= downcast::<bool>(right)))
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
    if type_check::<f64>(&left, &right) {
        Ok(Box::new(downcast::<f64>(left) < downcast::<f64>(right)))
    } else if type_check::<String>(&left, &right) {
        Ok(Box::new(
            downcast::<String>(left) < downcast::<String>(right),
        ))
    } else if type_check::<bool>(&left, &right) {
        Ok(Box::new(!downcast::<bool>(left) & downcast::<bool>(right)))
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
    if type_check::<f64>(&left, &right) {
        Ok(Box::new(downcast::<f64>(left) <= downcast::<f64>(right)))
    } else if type_check::<String>(&left, &right) {
        Ok(Box::new(
            downcast::<String>(left) <= downcast::<String>(right),
        ))
    } else if type_check::<bool>(&left, &right) {
        Ok(Box::new(downcast::<bool>(left) <= downcast::<bool>(right)))
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

    Ok(Box::new(None::<()>))
}

/// 変数定義評価
///
/// # Arguments
/// * `i` - 変数名
/// * `right` - 初期化式
///
/// # Return
/// * EvalResult - 評価後の値
fn var_decl(i: &String, right: Operand, env: &mut Environment) -> EvalResult {
    let value = to_env_value(right);
    env.push(i.to_string(), value);

    Ok(Box::new(None::<()>))
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
            Value::F64(f) => Ok(Box::new(*f)),
            Value::String(s) => Ok(Box::new(s.to_string())),
            Value::Bool(b) => Ok(Box::new(*b)),
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

        Ok(Box::new(None::<()>))
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
    if one_type_check::<String>(&operand) {
        Value::String(downcast::<String>(operand))
    } else if one_type_check::<f64>(&operand) {
        Value::F64(downcast::<f64>(operand))
    } else {
        Value::Bool(downcast::<bool>(operand))
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
    let mut ret: EvalResult = Ok(Box::new(None::<()>));
    for ast in ast_arr {
        ret = eval(ast, env);
    }

    ret
}

/// オペランド型チェック
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * bool
fn type_check<T: 'static>(left: &Operand, right: &Operand) -> bool {
    one_type_check::<T>(left) && one_type_check::<T>(right)
}

/// オペランド型チェック
///
/// # Arguments
/// * `operand` - オペランド
///
/// # Return
/// * bool
fn one_type_check<T: 'static>(operand: &Operand) -> bool {
    (**operand).type_id() == TypeId::of::<T>()
}

/// ダウンキャスト
///
/// # Arguments
/// * `operand` - オペランド
///
/// # Return
/// * f64/String/book - ダウンキャスト後のvalue
fn downcast<T: 'static>(operand: Operand) -> T {
    *operand.downcast::<T>().unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn リテラル_eval() {
        let ast = AstType::Number(1.0);
        let mut env = Environment::new();
        assert_eq!(
            1.0,
            *eval(&ast, &mut env).unwrap().downcast::<f64>().unwrap()
        );

        let ast = AstType::String("test".to_string());
        let mut env = Environment::new();
        assert_eq!(
            "test",
            *eval(&ast, &mut env).unwrap().downcast::<String>().unwrap()
        );

        let ast = AstType::True;
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::False;
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Nil;
        let mut env = Environment::new();
        assert_eq!(
            None,
            *eval(&ast, &mut env)
                .unwrap()
                .downcast::<Option::<()>>()
                .unwrap()
        );
    }

    #[test]
    fn 加算_eval() {
        let ast = AstType::Plus(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert_eq!(
            3.0,
            *eval(&ast, &mut env).unwrap().downcast::<f64>().unwrap()
        );

        let ast = AstType::Plus(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Plus(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(3.0)),
            )),
        );
        let mut env = Environment::new();
        assert_eq!(
            6.0,
            *eval(&ast, &mut env).unwrap().downcast::<f64>().unwrap()
        );
    }

    #[test]
    fn 減算_eval() {
        let ast = AstType::Minus(
            Box::new(AstType::Number(3.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert_eq!(
            1.0,
            *eval(&ast, &mut env).unwrap().downcast::<f64>().unwrap()
        );

        let ast = AstType::Minus(
            Box::new(AstType::Minus(
                Box::new(AstType::Number(10.0)),
                Box::new(AstType::Number(3.0)),
            )),
            Box::new(AstType::Number(1.0)),
        );
        let mut env = Environment::new();
        assert_eq!(
            6.0,
            *eval(&ast, &mut env).unwrap().downcast::<f64>().unwrap()
        );
    }

    #[test]
    fn 文字列連結_eval() {
        let ast = AstType::Plus(
            Box::new(AstType::String(String::from("test,"))),
            Box::new(AstType::String(String::from("hello"))),
        );
        let mut env = Environment::new();
        assert_eq!(
            "test,hello",
            *eval(&ast, &mut env).unwrap().downcast::<String>().unwrap()
        );
    }

    #[test]
    fn 積算_eval() {
        let ast = AstType::Mul(
            Box::new(AstType::Number(3.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert_eq!(
            6.0,
            *eval(&ast, &mut env).unwrap().downcast::<f64>().unwrap()
        );

        let ast = AstType::Mul(
            Box::new(AstType::Mul(
                Box::new(AstType::Number(10.0)),
                Box::new(AstType::Number(3.0)),
            )),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert_eq!(
            60.0,
            *eval(&ast, &mut env).unwrap().downcast::<f64>().unwrap()
        );
    }

    #[test]
    fn 除算_eval() {
        let ast = AstType::Div(
            Box::new(AstType::Number(6.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert_eq!(
            3.0,
            *eval(&ast, &mut env).unwrap().downcast::<f64>().unwrap()
        );

        let ast = AstType::Div(
            Box::new(AstType::Div(
                Box::new(AstType::Number(30.0)),
                Box::new(AstType::Number(3.0)),
            )),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert_eq!(
            5.0,
            *eval(&ast, &mut env).unwrap().downcast::<f64>().unwrap()
        );
    }

    #[test]
    fn unary_minus_eval() {
        let ast = AstType::UnaryMinus(Box::new(AstType::Number(1.0)));
        let mut env = Environment::new();
        assert_eq!(
            -1.0,
            *eval(&ast, &mut env).unwrap().downcast::<f64>().unwrap()
        );

        let ast = AstType::UnaryMinus(Box::new(AstType::Plus(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(4.0)),
        )));
        let mut env = Environment::new();
        assert_eq!(
            -5.0,
            *eval(&ast, &mut env).unwrap().downcast::<f64>().unwrap()
        );
    }

    #[test]
    fn unary_bang_eval() {
        let ast = AstType::Bang(Box::new(AstType::Number(1.0)));
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Bang(Box::new(AstType::Nil));
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Bang(Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Bang(Box::new(AstType::False));
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Bang(Box::new(AstType::String(String::from("a"))));
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn equal_equal_eval() {
        let ast = AstType::EqualEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(1.0)),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test"))),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test, test"))),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(Box::new(AstType::True), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(Box::new(AstType::False), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(Box::new(AstType::False), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn bang_equal_eval() {
        let ast = AstType::BangEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(1.0)),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test"))),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test, test"))),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(Box::new(AstType::True), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(Box::new(AstType::False), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(Box::new(AstType::False), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn greater_eval() {
        let ast = AstType::Greater(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(Box::new(AstType::True), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(Box::new(AstType::False), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(Box::new(AstType::False), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(Box::new(AstType::True), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn less_eval() {
        let ast = AstType::Less(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(Box::new(AstType::True), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(Box::new(AstType::False), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(Box::new(AstType::False), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(Box::new(AstType::True), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn greater_equal_eval() {
        let ast = AstType::GreaterEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("a"))),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(Box::new(AstType::True), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(Box::new(AstType::False), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(Box::new(AstType::False), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(Box::new(AstType::True), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn less_equal_eval() {
        let ast = AstType::LessEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(2.0)),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("a"))),
        );
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(Box::new(AstType::True), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(Box::new(AstType::False), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(Box::new(AstType::False), Box::new(AstType::True));
        let mut env = Environment::new();
        assert!(*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(Box::new(AstType::True), Box::new(AstType::False));
        let mut env = Environment::new();
        assert!(!*eval(&ast, &mut env).unwrap().downcast::<bool>().unwrap());
    }
}
