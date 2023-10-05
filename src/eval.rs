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
        AstType::Plus(l, r) => {
            let l = eval(l);
            let r = eval(r);

            eval_plus(l, r)
        }
        AstType::Minus(l, r) => {
            let l = eval(l);
            let r = eval(r);

            eval_minus(l, r)
        }
        AstType::Mul(l, r) => {
            let l = eval(l);
            let r = eval(r);

            eval_mul(l, r)
        }
        AstType::Div(l, r) => {
            let l = eval(l);
            let r = eval(r);

            eval_div(l, r)
        }
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
fn eval_plus(left: Box<dyn Any>, right: Box<dyn Any>) -> Box<dyn Any> {
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
fn eval_minus(left: Box<dyn Any>, right: Box<dyn Any>) -> Box<dyn Any> {
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
fn eval_mul(left: Box<dyn Any>, right: Box<dyn Any>) -> Box<dyn Any> {
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
fn eval_div(left: Box<dyn Any>, right: Box<dyn Any>) -> Box<dyn Any> {
    if (*left).type_id() == TypeId::of::<f64>() {
        Box::new(*left.downcast::<f64>().unwrap() / *right.downcast::<f64>().unwrap())
    } else {
        panic!("AstType::Mul Support Only Number!")
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
}
