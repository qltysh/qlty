mod child;

#[cfg(test)]
#[path = "child_test.rs"]
mod tests;

fn main() {
child::run();
}
