// A comment before the attributes stays.
/// Outer doc on the test module.
#[cfg(test)]
// A plain comment between attribute and item goes with the span.
#[allow(unused)]
mod r#tests {
    #[test]
    fn works() {}
}

/** Block doc on a raw-named module. */
mod r#test {
    fn inner() {}
}
/*! inner block doc stays */
pub fn production() -> i32 {
    1
}
