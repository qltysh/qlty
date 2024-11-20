class Foo {
  bar: string = "";
  baz: string = "";
}

function doSomething() {
  let foo = new Foo();
  foo.bar = "Hello";
  foo.baz = "World";
  return foo.bar + foo.baz;
}
