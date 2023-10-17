use crate::environment::{Environment, Value};

// 組み込み関数登録
pub fn register_func(env: &Environment) -> Environment {
    let mut env = env.clone();

    env.define(
        "clock".to_string(),
        Value::EmbeddedFunc(crate::embedded::func::clock),
    );

    env
}

fn clock() {
    println!("called clock");
}
