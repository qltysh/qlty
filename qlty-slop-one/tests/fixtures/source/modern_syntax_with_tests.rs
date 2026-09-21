unsafe extern "C" {
    fn c_function(value: i32) -> i32;
}

pub fn raw_address(value: &i32) -> *const i32 {
    &raw const *value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_address_points_at_value() {
        let value = 1;
        assert_eq!(unsafe { *raw_address(&value) }, 1);
    }
}
