using System;

public class FileComplexity
{
    public static void Main(string[] args)
    {
        int foo = 42;
        
        if (foo > 0)
        {
            if (foo > 10)
            {
                if (foo < 20)
                {
                    if (foo % 2 == 0)
                    {
                        if (foo % 3 == 0)
                        {
                            if (foo % 5 == 0)
                            {
                                if (foo % 7 == 0)
                                {
                                    if (foo % 11 == 0)
                                    {
                                        if (foo % 13 == 0)
                                        {
                                            if (foo % 17 == 0)
                                            {
                                                Console.WriteLine("Nested!");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
