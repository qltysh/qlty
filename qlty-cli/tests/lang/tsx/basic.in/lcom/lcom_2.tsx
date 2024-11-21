// lcom=2

class KlassA {
  constructor() {}

  foo(): any {
    return this.aaa;
  }

  bar(): any {
    return this.bbb;
  }
}

class KlassB {
  constructor() {}

  foo(): any {
    return this.baz;
  }

  bar(): any {
    return this.baz;
  }
}
