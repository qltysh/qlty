function not_nested(foo, bar): void {
  if ((foo === "cat" && bar === "dog") || (foo === "dog" && bar === "cat")) {
    console.log("Got a cat and a dog!");
  } else {
    console.log("Got nothing");
  }
}

function f0(): void {
  if (bar) {
    if (baz) {
      if (qux) {
        if (quux) {
          console.log("Not deeply nested enough!");
        }
      }
    }
  }
}

function f1(): void {
  if (bar) {
    if (baz) {
      if (qux) {
        if (quux) {
          console.log("Deeply nested!");
        }
      }
    }
  }
}

function f2(foo): string {
  switch (foo) {
    case 1:
      return "bar1";
    case 2:
      return "bar2";
    case 3:
      return "bar3";
    case 4:
      return "bar4";
    case 5:
      return "bar5";
    case 6:
      return "bar6";
    case 7:
      return "bar7";
    case 8:
      return "bar8";
    case 9:
      return "bar9";
    case 10:
      return "bar10";
    default:
      throw new Error("Invalid foo value");
  }
}
