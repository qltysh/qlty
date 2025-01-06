using System;

public class Main
{
    public static void NotNested(string foo, string bar)
    {
        if ((foo == "cat" && bar == "dog") || (foo == "dog" && bar == "cat"))
        {
            Console.WriteLine("Got a cat and a dog!");
        }
        else
        {
            Console.WriteLine("Got nothing");
        }
    }

    public static void F0(bool bar, bool baz, bool qux, bool quux)
    {
        if (bar)
        {
            if (baz)
            {
                if (qux)
                {
                    if (quux)
                    {
                        Console.WriteLine("Not deeply nested enough!");
                    }
                }
            }
        }
    }

    public static string F2(int foo)
    {
        switch (foo)
        {
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
                throw new ArgumentException("Invalid foo value");
        }
    }
}
