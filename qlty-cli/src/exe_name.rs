// Get the executable name by accessing argv[0] instead of using std::env::current_exe().
// This allows us to avoid relying on `/proc/self/exe` on Linux, which may not be available in all environments.
pub fn get_exe_name() -> String {
    std::env::args()
        .next()
        .expect("Unable to identify current executable")
}
