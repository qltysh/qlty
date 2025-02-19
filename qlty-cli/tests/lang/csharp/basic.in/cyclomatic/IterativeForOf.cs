using System;

namespace Cyclomatic
{
  class IterativeForOf {
    public static void Main(string[] args) {
      foreach (var animal in new string[] { "dog", "cat", "bear" }) {
        Console.WriteLine(animal);
      }
    }
  }
}