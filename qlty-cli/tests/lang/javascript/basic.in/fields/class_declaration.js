class Foo {
  constructor() {
    this.bar = "";
    this.baz = "";
  }
}

function doSomething() {
  let foo = new Foo();
  foo.bar = "Hello";
  foo.baz = "World";
  return foo.bar + foo.baz;
}
