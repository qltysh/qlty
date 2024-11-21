class Foo {
  constructor() {
    this.bar = "";
    this.baz = "";
  }

  doSomething(baz, bar) {
    this.bar = bar;
    this.baz = baz;
    return this.bar + this.baz;
  }
}
