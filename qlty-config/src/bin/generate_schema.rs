use qlty_config::QltyConfig;
use schemars::schema_for;

fn main() {
    let schema = schema_for!(QltyConfig);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
