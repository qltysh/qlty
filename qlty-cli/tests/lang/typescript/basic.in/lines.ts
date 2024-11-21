function f1(): any {
  const sourceFile = new Tempfile(["ruby", ".kt"]);
  sourceFile.write("foo(...args)");
  const tree = sourceFile.parse();

  bar();
}

// Foo
function f2(): any {
  bar(); // does not count as comment line
}

// multi-line comment
/*

line1
line2

line4
*/

function f3(): any {
  bar();
}
