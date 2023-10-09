use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    F64(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
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

    pub fn push(&mut self, key: String, value: Value) -> Option<Value> {
        self.variables.insert(key, value)
        //if self.enclosing.is_some() {
        //    self.enclosing.as_mut().unwrap().push(key, value)
        //} else {
        //    self.variables.insert(key, value)
        //}
    }

    pub fn get(&self, key: &String) -> Option<&Value> {
        let result = self.variables.get(key);

        if result.is_some() {
            result
        } else {
            self.enclosing.as_ref().unwrap().get(key)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn 環境テスト() {
        let mut env = Environment::new();
        env.push("a".to_string(), Value::F64(1.0));
        let val = env.get(&"a".to_string());
        assert!(val.is_some());
    }
}