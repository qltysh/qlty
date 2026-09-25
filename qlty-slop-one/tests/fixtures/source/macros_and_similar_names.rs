// mod tests { this is a comment }
macro_rules! fixture { () => { mod tests { fn generated() {} } }; }
fixture! { mod test { fn supplied() {} } }
mod tests;
mod test;
mod test_support { pub fn helper() {} }
mod testsuite { pub fn run() {} }
#[cfg(test)]
mod tests_helpers { pub fn helper() {} }
