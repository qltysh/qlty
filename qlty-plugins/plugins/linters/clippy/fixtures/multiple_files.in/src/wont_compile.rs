// A version of main.rs that wont compile because its missing ; after mod.
// Used to verify compile errors are properly reported as linter failures.
mod high

fn main() {
    let x = 1;
    let y = 2;
    if x == y || x < y {}
    println!("Hello World");
}
