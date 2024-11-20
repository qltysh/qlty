class Foo {
  bar: string;
  baz: string;

  constructor() {
    this.bar = "";
    this.baz = "";
  }

  doSomething(baz: string, bar: string): string {
    this.bar = bar;
    this.baz = baz;
    return this.bar + this.baz;
  }
}
