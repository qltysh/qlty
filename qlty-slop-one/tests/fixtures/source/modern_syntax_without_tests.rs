unsafe extern "C" {
    fn c_function(value: i32) -> i32;
}

pub fn raw_address(value: &i32) -> *const i32 {
    &raw const *value
}

pub fn call(value: i32) -> i32 {
    unsafe { c_function(value) }
}
