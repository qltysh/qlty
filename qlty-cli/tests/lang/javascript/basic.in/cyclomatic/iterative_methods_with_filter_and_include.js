function foo() {
  let animals = ["dog", "cat", "bear", "tiger"];

  animals
    .filter((animal) => {
      return animal.length > 3;
    })
    .forEach((animal) => {
      console.log(animal);
    });

  if (animals.includes("cat")) {
    console.log("Found a cat!");
  }
}
