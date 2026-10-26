#[allow(dead_code)] fn keep() {} #[cfg(test)] mod tests { fn gone() {} } fn later() {}
mod parent { mod r#test { fn gone() {} } pub fn keep() {} }
fn final_item() {}
