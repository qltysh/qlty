using System;
using System.IO;

public class Main
{
    public static void F1()
    {
        try
        {
            string tempFilePath = Path.Combine(
                Path.GetTempPath(),
                $"ruby{Guid.NewGuid()}.kt"
            );
            
            using (var writer = new StreamWriter(tempFilePath))
            {
                writer.Write("foo(...args)");
            }

            object tree = ParseFile(tempFilePath);

            Bar();
        }
        catch (IOException e)
        {
            Console.Error.WriteLine(e);
        }
    }

    public static object ParseFile(string filePath)
    {
        return new object();
    }

    // Foo
    public static void F2()
    {
        Bar(); // does not count as comment line
    }

    // multi-line comment
    /*
     * line1
     * line2
     * line4
     */

    public static void F3()
    {
        Bar();
    }

    public static void Bar()
    {
        Console.WriteLine("bar() called");
    }
}
