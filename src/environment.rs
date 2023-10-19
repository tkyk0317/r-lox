use crate::ast::AstType;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    F64(f64),
    String(String),
    Bool(bool),
    UserFunc(Vec<AstType>, AstType),
    EmbeddedFunc(fn()), // TODO: 可変長引数に対応したい
}

#[derive(Debug, Clone)]
pub struct Environment {
    pub enclosing: Option<Box<Environment>>,
    variables: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            variables: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: Environment) -> Self {
        let mut instance = Self::new();
        instance.enclosing = Some(Box::new(enclosing));

        instance
    }

    pub fn define(&mut self, key: String, value: Value) -> Option<Value> {
        self.variables.insert(key, value)
    }

    pub fn push(&mut self, key: String, value: Value) -> Option<Value> {
        if self.variables.get(&key).is_some() {
            self.variables.insert(key, value)
        } else if self.enclosing.is_some() {
            self.enclosing.as_mut().unwrap().push(key, value)
        } else {
            None
        }
    }

    pub fn get(&self, key: &String) -> Option<&Value> {
        self.variables
            .get(key)
            .or_else(|| self.enclosing.as_ref().unwrap().get(key))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn 環境テスト() {
        let mut env = Environment::new();
        env.define("a".to_string(), Value::F64(1.0));
        let val = env.get(&"a".to_string());
        assert!(val.is_some());
        assert_eq!(&Value::F64(1.0), val.unwrap());

        let mut block_env = Environment::with_enclosing(env.clone());
        block_env.define("a".to_string(), Value::F64(10.0));
        assert_eq!(&Value::F64(10.0), block_env.get(&"a".to_string()).unwrap());
    }
}
