function simple(): void {}

function complex(): void {
  let bar = 42;
  if (bar) {
    if (bar > 10) {
      if (bar < 20) {
        if (bar % 2 === 0) {
          if (bar % 3 === 0) {
            console.log("Nested!");
          }
        }
      }
    }
  }
}
