// lcom = 0 for all the tests in this file, totalling 0

function foo() {
  return this.bar;
}

class Klass {
  constructor() {}

  foo() {
    return null;
  }
}

class Klass {
  constructor() {}

  foo() {
    return bar.baz;
  }
}
