using System;
using System.Collections.Generic;
using System.Linq;

namespace Cyclomatic
{
  class IterativeMethodsWithFilterAndInclude {
    public static void Main(string[] args) {
      List<string> animals = new List<string> { "dog", "cat", "bear", "tiger" };

      animals.Where(animal => animal.Length > 3)
             .ToList()
             .ForEach(animal => Console.WriteLine(animal));

      if (animals.Contains("cat")) {
        Console.WriteLine("Found a cat!");
      }
    }
  }
}