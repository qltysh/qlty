using System;
using System.Collections.Generic;
using System.Linq;

namespace Cyclomatic
{
  class IterativeMap {
    public static void Main(string[] args) {
      List<string> animals = new List<string> { "dog", "cat", "bear" };

      animals.Select(animal => {
        Console.WriteLine(animal);
        return animal;
      }).ToList();
    }
  }
}