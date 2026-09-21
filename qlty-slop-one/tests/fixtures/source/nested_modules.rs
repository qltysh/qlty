//! crate docs
pub mod production {
    pub fn before() -> &'static str { r###"mod tests { }"### }
    /// Unit tests, with Unicode: λ
    #[cfg(all(test, feature = "testing"))]
    #[allow(dead_code)]
    pub(crate) mod tests {
        /* nested /* } */ comment */
        const TEXT: &str = r##"} mod test {"##;
        #[test] fn example() { assert_eq!('}', '}'); }
        mod test { fn inner() {} }
    }
    pub fn after() {}
}
mod test { fn another() {} }
pub fn last() {}
