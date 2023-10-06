use crate::ast::AstType;
use std::any::{Any, TypeId};

/// AST評価
///
/// # Arguments
/// * `ast` - AST
pub fn eval(ast: &AstType) -> Box<dyn Any> {
    match ast {
        AstType::True => Box::new(true),
        AstType::False => Box::new(false),
        AstType::Nil => Box::new(None::<()>),
        AstType::Number(n) => Box::new(*n),
        AstType::String(s) => Box::new(s.clone()),
        AstType::Bang(o) => bang(eval(o)),
        AstType::UnaryMinus(o) => unary_minus(eval(o)),
        AstType::Plus(l, r) => plus(eval(l), eval(r)),
        AstType::Minus(l, r) => minus(eval(l), eval(r)),
        AstType::Mul(l, r) => mul(eval(l), eval(r)),
        AstType::Div(l, r) => div(eval(l), eval(r)),
        AstType::EqualEqual(l, r) => equal_equal(eval(l), eval(r)),
        AstType::BangEqual(l, r) => bang_equal(eval(l), eval(r)),
        AstType::Greater(l, r) => greater(eval(l), eval(r)),
        AstType::Less(l, r) => less(eval(l), eval(r)),
        AstType::GreaterEqual(l, r) => greater_equal(eval(l), eval(r)),
        AstType::LessEqual(l, r) => less_equal(eval(l), eval(r)),
        _ => panic!(""),
    }
}

/// 評価結果出力
pub fn print(result: Box<dyn Any>) {
    if (*result).type_id() == TypeId::of::<f64>() {
        println!("{}", result.downcast::<f64>().unwrap())
    } else {
        println!("{}", result.downcast::<String>().unwrap())
    }
}

/// プラス演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * Box<dyn Any> - 評価後の値（f64 or String）
fn plus(left: Box<dyn Any>, right: Box<dyn Any>) -> Box<dyn Any> {
    if (*left).type_id() == TypeId::of::<f64>() {
        Box::new(*left.downcast::<f64>().unwrap() + *right.downcast::<f64>().unwrap())
    } else if (*left).type_id() == TypeId::of::<String>() {
        Box::new(format!(
            "{}{}",
            *left.downcast::<String>().unwrap(),
            *right.downcast::<String>().unwrap()
        ))
    } else {
        panic!("Not Support Operand: only support f64 or String")
    }
}

/// マイナス演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * Box<dyn Any> - 評価後の値（f64）
fn minus(left: Box<dyn Any>, right: Box<dyn Any>) -> Box<dyn Any> {
    if (*left).type_id() == TypeId::of::<f64>() {
        Box::new(*left.downcast::<f64>().unwrap() - *right.downcast::<f64>().unwrap())
    } else {
        panic!("AstType::Minus Support Only Number!")
    }
}

/// 積算演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * Box<dyn Any> - 評価後の値（f64）
fn mul(left: Box<dyn Any>, right: Box<dyn Any>) -> Box<dyn Any> {
    if (*left).type_id() == TypeId::of::<f64>() {
        Box::new(*left.downcast::<f64>().unwrap() * *right.downcast::<f64>().unwrap())
    } else {
        panic!("AstType::Mul Support Only Number!")
    }
}

/// 除算演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * Box<dyn Any> - 評価後の値（f64）
fn div(left: Box<dyn Any>, right: Box<dyn Any>) -> Box<dyn Any> {
    if (*left).type_id() == TypeId::of::<f64>() {
        Box::new(*left.downcast::<f64>().unwrap() / *right.downcast::<f64>().unwrap())
    } else {
        panic!("AstType::Mul Support Only Number!")
    }
}

/// -演算子評価
///
/// # Arguments
/// * `operand` - オペランド
///
/// # Return
/// * Box<dyn Any> - 評価後の値（f64）
fn unary_minus(operand: Box<dyn Any>) -> Box<dyn Any> {
    if (*operand).type_id() == TypeId::of::<f64>() {
        Box::new(-(*operand.downcast::<f64>().unwrap()))
    } else {
        panic!("AstType::UnaryMinus Support Only Number!")
    }
}

/// !演算子評価
///
/// # Arguments
/// * `operand` - オペランド
///
/// # Return
/// * Box<dyn Any> - 評価後の値（bool）
fn bang(operand: Box<dyn Any>) -> Box<dyn Any> {
    if (*operand).type_id() == TypeId::of::<bool>() {
        Box::new(!*operand.downcast::<bool>().unwrap())
    } else if (*operand).type_id() == TypeId::of::<Option<()>>() {
        Box::new(true)
    } else {
        Box::new(false)
    }
}

/// ==演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * Box<dyn Any> - 評価後の値（bool）
fn equal_equal(left: Box<dyn Any>, right: Box<dyn Any>) -> Box<dyn Any> {
    if (*left).type_id() == TypeId::of::<f64>() {
        Box::new(*left.downcast::<f64>().unwrap() == *right.downcast::<f64>().unwrap())
    } else if (*left).type_id() == TypeId::of::<String>() {
        Box::new(*left.downcast::<String>().unwrap() == *right.downcast::<String>().unwrap())
    } else if (*left).type_id() == TypeId::of::<bool>() {
        Box::new(*left.downcast::<bool>().unwrap() == *right.downcast::<bool>().unwrap())
    } else {
        panic!("AstType::EqualEqual Support Only Number!")
    }
}

/// !=演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * Box<dyn Any> - 評価後の値（bool）
fn bang_equal(left: Box<dyn Any>, right: Box<dyn Any>) -> Box<dyn Any> {
    let ret = equal_equal(left, right);

    Box::new(!*ret.downcast::<bool>().unwrap())
}

/// >演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * Box<dyn Any> - 評価後の値（bool）
fn greater(left: Box<dyn Any>, right: Box<dyn Any>) -> Box<dyn Any> {
    if (*left).type_id() == TypeId::of::<f64>() {
        Box::new(*left.downcast::<f64>().unwrap() > *right.downcast::<f64>().unwrap())
    } else if (*left).type_id() == TypeId::of::<String>() {
        Box::new(*left.downcast::<String>().unwrap() > *right.downcast::<String>().unwrap())
    } else if (*left).type_id() == TypeId::of::<bool>() {
        Box::new(*left.downcast::<bool>().unwrap() & !(*right.downcast::<bool>().unwrap()))
    } else {
        panic!("AstType::Greater Support Only Number!")
    }
}

/// >=演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * Box<dyn Any> - 評価後の値（bool）
fn greater_equal(left: Box<dyn Any>, right: Box<dyn Any>) -> Box<dyn Any> {
    if (*left).type_id() == TypeId::of::<f64>() {
        Box::new(*left.downcast::<f64>().unwrap() >= *right.downcast::<f64>().unwrap())
    } else if (*left).type_id() == TypeId::of::<String>() {
        Box::new(*left.downcast::<String>().unwrap() >= *right.downcast::<String>().unwrap())
    } else if (*left).type_id() == TypeId::of::<bool>() {
        Box::new(*left.downcast::<bool>().unwrap() >= *right.downcast::<bool>().unwrap())
    } else {
        panic!("AstType::Greater Support Only Number!")
    }
}

/// <演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * Box<dyn Any> - 評価後の値（bool）
fn less(left: Box<dyn Any>, right: Box<dyn Any>) -> Box<dyn Any> {
    if (*left).type_id() == TypeId::of::<f64>() {
        Box::new(*left.downcast::<f64>().unwrap() < *right.downcast::<f64>().unwrap())
    } else if (*left).type_id() == TypeId::of::<String>() {
        Box::new(*left.downcast::<String>().unwrap() < *right.downcast::<String>().unwrap())
    } else if (*left).type_id() == TypeId::of::<bool>() {
        Box::new(!(*left.downcast::<bool>().unwrap()) & *right.downcast::<bool>().unwrap())
    } else {
        panic!("AstType::Less Support Only Number!")
    }
}

/// <=演算子評価
///
/// # Arguments
/// * `left` - 左オペランド
/// * `right` - 右オペランド
///
/// # Return
/// * Box<dyn Any> - 評価後の値（bool）
fn less_equal(left: Box<dyn Any>, right: Box<dyn Any>) -> Box<dyn Any> {
    if (*left).type_id() == TypeId::of::<f64>() {
        Box::new(*left.downcast::<f64>().unwrap() <= *right.downcast::<f64>().unwrap())
    } else if (*left).type_id() == TypeId::of::<String>() {
        Box::new(*left.downcast::<String>().unwrap() <= *right.downcast::<String>().unwrap())
    } else if (*left).type_id() == TypeId::of::<bool>() {
        Box::new(*left.downcast::<bool>().unwrap() <= *right.downcast::<bool>().unwrap())
    } else {
        panic!("AstType::Less Support Only Number!")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn リテラル_eval() {
        let ast = AstType::Number(1.0);
        assert_eq!(1.0, *eval(&ast).downcast::<f64>().unwrap());

        let ast = AstType::String("test".to_string());
        assert_eq!("test", *eval(&ast).downcast::<String>().unwrap());

        let ast = AstType::True;
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::False;
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Nil;
        assert_eq!(None, *eval(&ast).downcast::<Option::<()>>().unwrap());
    }

    #[test]
    fn 加算_eval() {
        let ast = AstType::Plus(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert_eq!(3.0, *eval(&ast).downcast::<f64>().unwrap());

        let ast = AstType::Plus(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Plus(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(3.0)),
            )),
        );
        assert_eq!(6.0, *eval(&ast).downcast::<f64>().unwrap());
    }

    #[test]
    fn 減算_eval() {
        let ast = AstType::Minus(
            Box::new(AstType::Number(3.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert_eq!(1.0, *eval(&ast).downcast::<f64>().unwrap());

        let ast = AstType::Minus(
            Box::new(AstType::Minus(
                Box::new(AstType::Number(10.0)),
                Box::new(AstType::Number(3.0)),
            )),
            Box::new(AstType::Number(1.0)),
        );
        assert_eq!(6.0, *eval(&ast).downcast::<f64>().unwrap());
    }

    #[test]
    fn 文字列連結_eval() {
        let ast = AstType::Plus(
            Box::new(AstType::String(String::from("test,"))),
            Box::new(AstType::String(String::from("hello"))),
        );
        assert_eq!("test,hello", *eval(&ast).downcast::<String>().unwrap());
    }

    #[test]
    fn 積算_eval() {
        let ast = AstType::Mul(
            Box::new(AstType::Number(3.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert_eq!(6.0, *eval(&ast).downcast::<f64>().unwrap());

        let ast = AstType::Mul(
            Box::new(AstType::Mul(
                Box::new(AstType::Number(10.0)),
                Box::new(AstType::Number(3.0)),
            )),
            Box::new(AstType::Number(2.0)),
        );
        assert_eq!(60.0, *eval(&ast).downcast::<f64>().unwrap());
    }

    #[test]
    fn 除算_eval() {
        let ast = AstType::Div(
            Box::new(AstType::Number(6.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert_eq!(3.0, *eval(&ast).downcast::<f64>().unwrap());

        let ast = AstType::Div(
            Box::new(AstType::Div(
                Box::new(AstType::Number(30.0)),
                Box::new(AstType::Number(3.0)),
            )),
            Box::new(AstType::Number(2.0)),
        );
        assert_eq!(5.0, *eval(&ast).downcast::<f64>().unwrap());
    }

    #[test]
    fn unary_minus_eval() {
        let ast = AstType::UnaryMinus(Box::new(AstType::Number(1.0)));
        assert_eq!(-1.0, *eval(&ast).downcast::<f64>().unwrap());

        let ast = AstType::UnaryMinus(Box::new(AstType::Plus(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(4.0)),
        )));
        assert_eq!(-5.0, *eval(&ast).downcast::<f64>().unwrap());
    }

    #[test]
    fn unary_bang_eval() {
        let ast = AstType::Bang(Box::new(AstType::Number(1.0)));
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Bang(Box::new(AstType::Nil));
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Bang(Box::new(AstType::True));
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Bang(Box::new(AstType::False));
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Bang(Box::new(AstType::String(String::from("a"))));
        assert!(!*eval(&ast).downcast::<bool>().unwrap());
    }

    #[test]
    fn equal_equal_eval() {
        let ast = AstType::EqualEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(1.0)),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test"))),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test, test"))),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(Box::new(AstType::True), Box::new(AstType::True));
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(Box::new(AstType::False), Box::new(AstType::False));
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::EqualEqual(Box::new(AstType::False), Box::new(AstType::True));
        assert!(!*eval(&ast).downcast::<bool>().unwrap());
    }

    #[test]
    fn bang_equal_eval() {
        let ast = AstType::BangEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(1.0)),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test"))),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(
            Box::new(AstType::String(String::from("test"))),
            Box::new(AstType::String(String::from("test, test"))),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(Box::new(AstType::True), Box::new(AstType::True));
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(Box::new(AstType::False), Box::new(AstType::False));
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::BangEqual(Box::new(AstType::False), Box::new(AstType::True));
        assert!(*eval(&ast).downcast::<bool>().unwrap());
    }

    #[test]
    fn greater_eval() {
        let ast = AstType::Greater(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Greater(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Greater(Box::new(AstType::True), Box::new(AstType::True));
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Greater(Box::new(AstType::False), Box::new(AstType::False));
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Greater(Box::new(AstType::False), Box::new(AstType::True));
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Greater(Box::new(AstType::True), Box::new(AstType::False));
        assert!(*eval(&ast).downcast::<bool>().unwrap());
    }

    #[test]
    fn less_eval() {
        let ast = AstType::Less(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Less(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Less(Box::new(AstType::True), Box::new(AstType::True));
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Less(Box::new(AstType::False), Box::new(AstType::False));
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Less(Box::new(AstType::False), Box::new(AstType::True));
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::Less(Box::new(AstType::True), Box::new(AstType::False));
        assert!(!*eval(&ast).downcast::<bool>().unwrap());
    }

    #[test]
    fn greater_equal_eval() {
        let ast = AstType::GreaterEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("a"))),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(Box::new(AstType::True), Box::new(AstType::True));
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(Box::new(AstType::False), Box::new(AstType::False));
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(Box::new(AstType::False), Box::new(AstType::True));
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::GreaterEqual(Box::new(AstType::True), Box::new(AstType::False));
        assert!(*eval(&ast).downcast::<bool>().unwrap());
    }

    #[test]
    fn less_equal_eval() {
        let ast = AstType::LessEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(1.0)),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::Number(1.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::Number(2.0)),
            Box::new(AstType::Number(2.0)),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("b"))),
            Box::new(AstType::String(String::from("a"))),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("b"))),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("bc"))),
            Box::new(AstType::String(String::from("ab"))),
        );
        assert!(!*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("ba"))),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(
            Box::new(AstType::String(String::from("a"))),
            Box::new(AstType::String(String::from("a"))),
        );
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(Box::new(AstType::True), Box::new(AstType::True));
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(Box::new(AstType::False), Box::new(AstType::False));
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(Box::new(AstType::False), Box::new(AstType::True));
        assert!(*eval(&ast).downcast::<bool>().unwrap());

        let ast = AstType::LessEqual(Box::new(AstType::True), Box::new(AstType::False));
        assert!(!*eval(&ast).downcast::<bool>().unwrap());
    }
}
