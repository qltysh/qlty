function f0(): void {
  let animals: string[] = ["dog", "cat", "bear", "tiger"];

  animals
    .filter((animal: string) => {
      return animal.length > 3;
    })
    .forEach((animal: string) => {
      console.log(animal);
    });

  if (animals.includes("cat")) {
    console.log("Found a cat!");
  }
}
