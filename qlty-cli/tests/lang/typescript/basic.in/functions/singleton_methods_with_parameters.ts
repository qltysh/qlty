class Foo {
  static bar(dog: any, cat: any): string[] {
    return [dog, cat].map((animal) => animal.toString());
  }
}
