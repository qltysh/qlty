const foobar = () => { }
const barfoo = () => { }

enum Bar { Baz };

const foo = (bar: Bar) => {
  switch (bar) {
    case Bar.Baz:
      foobar();
      barfoo();
      break;
  }
  { !foo ? null : 1 }
}

enum Foo { Bae };
