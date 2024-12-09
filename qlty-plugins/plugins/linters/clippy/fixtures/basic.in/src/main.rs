// clippy/manual_non_exhaustive has {"suggested_replacement": null} in its JSON, so we want to
// explicitly test that. Note that a match {} block against this enum in the callgraph needs to
// be compiled to actually trigger it.
pub enum Gibberish {
    Foo(String),
    #[doc(hidden)]
    __Nonexhaustive,
}

fn main() {
    let x = 1;
    let y = 2;
    if x == y || x < y {}
    println!("Hello World");

    // empty format literal
    println!("{}", "empty format literal");

    // needless range loop
    let vec = vec!['a', 'b', 'c'];
    for i in 0..vec.len() {
        println!("{}", vec[i]);

    }
    // manual nonexhaustive
    let z = Gibberish::Foo("fizz".to_string());
    println!("gibberish is {}", match z {
        Gibberish::Foo(_) => "buzz",
        Gibberish::__Nonexhaustive => unreachable!(),
    });
}
