use crate::ast::AstType;
use std::any::{Any, TypeId};
use std::error;
use std::fmt;

/// ランタイムエラー
pub enum RuntimeError {
    OperandType(Operand),
    TwoOperandType(Operand, Operand),
}
impl RuntimeError {
    fn operand_type(&self, operand: &Operand) -> Option<&str> {
        if (**operand).type_id() == TypeId::of::<String>() {
            Some("String")
        } else if (**operand).type_id() == TypeId::of::<f64>() {
            Some("f64")
        } else if (**operand).type_id() == TypeId::of::<bool>() {
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

type Operand = Box<dyn Any>;
type EvalResult = Result<Operand, RuntimeError>;

/// AST評価
///
/// # Arguments
/// * `ast` - AST
pub fn eval(ast: &AstType) -> EvalResult {
    match ast {
        AstType::True => Ok(Box::new(true)),
        AstType::False => Ok(Box::new(false)),
        AstType::Nil => Ok(Box::new(None::<()>)),
        AstType::Number(n) => Ok(Box::new(*n)),
        AstType::String(s) => Ok(Box::new(s.clone())),
        AstType::Bang(o) => bang(eval(o)?),
        AstType::UnaryMinus(o) => unary_minus(eval(o)?),
        AstType::Plus(l, r) => plus(eval(l)?, eval(r)?),
        AstType::Minus(l, r) => minus(eval(l)?, eval(r)?),
        AstType::Mul(l, r) => mul(eval(l)?, eval(r)?),
        AstType::Div(l, r) => div(eval(l)?, eval(r)?),
        AstType::EqualEqual(l, r) => equal_equal(eval(l)?, eval(r)?),
        AstType::BangEqual(l, r) => bang_equal(eval(l)?, eval(r)?),
        AstType::Greater(l, r) => greater(eval(l)?, eval(r)?),
        AstType::Less(l, r) => less(eval(l)?, eval(r)?),
        AstType::GreaterEqual(l, r) => greater_equal(eval(l)?, eval(r)?),
        AstType::LessEqual(l, r) => less_equal(eval(l)?, eval(r)?),
        _ => panic!(""),
    }
}

/// 評価結果出力
pub fn print(result: Operand) {
    if (*result).type_id() == TypeId::of::<f64>() {
        println!("{}", downcast::<f64>(result))
    } else {
        println!("{}", downcast::<String>(result))
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
    if (*operand).type_id() == TypeId::of::<f64>() {
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
    if (*operand).type_id() == TypeId::of::<bool>() {
        Ok(Box::new(!*operand.downcast::<bool>().unwrap()))
    } else if (*operand).type_id() == TypeId::of::<Option<()>>() {
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
    if (*left).type_id() == TypeId::of::<f64>() {
        Ok(Box::new(
            *left.downcast::<f64>().unwrap() >= *right.downcast::<f64>().unwrap(),
        ))
    } else if (*left).type_id() == TypeId::of::<String>() {
        Ok(Box::new(
            *left.downcast::<String>().unwrap() >= *right.downcast::<String>().unwrap(),
        ))
    } else if (*left).type_id() == TypeId::of::<bool>() {
        Ok(Box::new(
            *left.downcast::<bool>().unwrap() >= *right.downcast::<bool>().unwrap(),
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
    if type_check::<f64>(&left, &right) {
        Ok(Box::new(
            *left.downcast::<f64>().unwrap() < *right.downcast::<f64>().unwrap(),
        ))
    } else if type_check::<String>(&left, &right) {
        Ok(Box::new(
            *left.downcast::<String>().unwrap() < *right.downcast::<String>().unwrap(),
        ))
    } else if type_check::<bool>(&left, &right) {
        Ok(Box::new(
            !(*left.downcast::<bool>().unwrap()) & *right.downcast::<bool>().unwrap(),
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

/// オペランド型チェック
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * bool
fn type_check<T: 'static>(left: &Operand, right: &Operand) -> bool {
    (**left).type_id() == TypeId::of::<T>() && (**right).type_id() == TypeId::of::<T>()
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
        assert_eq!(1.0, *eval(&ast).unwrap().downcast::<f64>().unwrap());

        let ast = AstType::String("test".to_string());
        assert_eq!("test", *eval(&ast).unwrap().downcast::<String>().unwrap());

        let ast = AstType::True;
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::False;
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Nil;
        assert_eq!(
            None,
            *eval(&ast).unwrap().downcast::<Option::<()>>().unwrap()
        );
    }

    #[test]
    fn 加算_eval() {
        let ast = AstType::Plus(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert_eq!(3.0, *eval(&ast).unwrap().downcast::<f64>().unwrap());

        let ast = AstType::Plus(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Plus(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(3.0)),
            )),
        );
        assert_eq!(6.0, *eval(&ast).unwrap().downcast::<f64>().unwrap());
    }

    #[test]
    fn 減算_eval() {
        let ast = AstType::Minus(
            Box::new(AstType::Number(3.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert_eq!(1.0, *eval(&ast).unwrap().downcast::<f64>().unwrap());

        let ast = AstType::Minus(
            Box::new(AstType::Minus(
                Box::new(AstType::Number(10.0)),
                Box::new(AstType::Number(3.0)),
            )),
            Box::new(AstType::Number(1.0)),
        );
        assert_eq!(6.0, *eval(&ast).unwrap().downcast::<f64>().unwrap());
    }

    #[test]
    fn 文字列連結_eval() {
        let ast = AstType::Plus(
            Box::new(AstType::String(String::from("test,"))),
            Box::new(AstType::String(String::from("hello"))),
        );
        assert_eq!(
            "test,hello",
            *eval(&ast).unwrap().downcast::<String>().unwrap()
        );
    }

    #[test]
    fn 積算_eval() {
        let ast = AstType::Mul(
            Box::new(AstType::Number(3.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert_eq!(6.0, *eval(&ast).unwrap().downcast::<f64>().unwrap());

        let ast = AstType::Mul(
            Box::new(AstType::Mul(
                Box::new(AstType::Number(10.0)),
                Box::new(AstType::Number(3.0)),
            )),
            Box::new(AstType::Number(2.0)),
        );
        assert_eq!(60.0, *eval(&ast).unwrap().downcast::<f64>().unwrap());
    }

    #[test]
    fn 除算_eval() {
        let ast = AstType::Div(
            Box::new(AstType::Number(6.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert_eq!(3.0, *eval(&ast).unwrap().downcast::<f64>().unwrap());

        let ast = AstType::Div(
            Box::new(AstType::Div(
                Box::new(AstType::Number(30.0)),
                Box::new(AstType::Number(3.0)),
            )),
            Box::new(AstType::Number(2.0)),
        );
        assert_eq!(5.0, *eval(&ast).unwrap().downcast::<f64>().unwrap());
    }

    #[test]
    fn unary_minus_eval() {
        let ast = AstType::UnaryMinus(Box::new(AstType::Number(1.0)));
        assert_eq!(-1.0, *eval(&ast).unwrap().downcast::<f64>().unwrap());

        let ast = AstType::UnaryMinus(Box::new(AstType::Plus(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(4.0)),
        )));
        assert_eq!(-5.0, *eval(&ast).unwrap().downcast::<f64>().unwrap());
    }

    #[test]
    fn unary_bang_eval() {
        let ast = AstType::Bang(Box::new(AstType::Number(1.0)));
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Bang(Box::new(AstType::Nil));
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Bang(Box::new(AstType::True));
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Bang(Box::new(AstType::False));
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Bang(Box::new(AstType::String(String::from("a"))));
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn equal_equal_eval() {
        let ast = AstType::EqualEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(1.0)),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test"))),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test, test"))),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(Box::new(AstType::True), Box::new(AstType::True));
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(Box::new(AstType::False), Box::new(AstType::False));
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(Box::new(AstType::False), Box::new(AstType::True));
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn bang_equal_eval() {
        let ast = AstType::BangEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(1.0)),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test"))),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test, test"))),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(Box::new(AstType::True), Box::new(AstType::True));
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(Box::new(AstType::False), Box::new(AstType::False));
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(Box::new(AstType::False), Box::new(AstType::True));
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn greater_eval() {
        let ast = AstType::Greater(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(Box::new(AstType::True), Box::new(AstType::True));
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(Box::new(AstType::False), Box::new(AstType::False));
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(Box::new(AstType::False), Box::new(AstType::True));
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Greater(Box::new(AstType::True), Box::new(AstType::False));
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn less_eval() {
        let ast = AstType::Less(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(Box::new(AstType::True), Box::new(AstType::True));
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(Box::new(AstType::False), Box::new(AstType::False));
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(Box::new(AstType::False), Box::new(AstType::True));
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::Less(Box::new(AstType::True), Box::new(AstType::False));
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn greater_equal_eval() {
        let ast = AstType::GreaterEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("a"))),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(Box::new(AstType::True), Box::new(AstType::True));
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(Box::new(AstType::False), Box::new(AstType::False));
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(Box::new(AstType::False), Box::new(AstType::True));
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(Box::new(AstType::True), Box::new(AstType::False));
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn less_equal_eval() {
        let ast = AstType::LessEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("a"))),
        );
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(Box::new(AstType::True), Box::new(AstType::True));
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(Box::new(AstType::False), Box::new(AstType::False));
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(Box::new(AstType::False), Box::new(AstType::True));
        assert!(*eval(&ast).unwrap().downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(Box::new(AstType::True), Box::new(AstType::False));
        assert!(!*eval(&ast).unwrap().downcast::<bool>().unwrap());
    }
}
