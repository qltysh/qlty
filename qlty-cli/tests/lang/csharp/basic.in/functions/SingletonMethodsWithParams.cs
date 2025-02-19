using System;
using System.Collections.Generic;
using System.Linq;

public class SingletonMethodsWithParams
{
    public static List<string> Bar(object dog, object cat)
    {
        return new List<object> { dog, cat }
            .Select(animal => animal.ToString())
            .ToList();
    }
}