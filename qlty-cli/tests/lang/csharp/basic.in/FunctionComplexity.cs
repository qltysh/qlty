using System;

public class FunctionComplexity
{
    public void Simple()
    {
    }

    public void Complex()
    {
        int bar = 42;

        if (bar > 0)
        {
            if (bar > 10)
            {
                if (bar < 20)
                {
                    if (bar % 2 == 0)
                    {
                        if (bar % 3 == 0)
                        {
                            Console.WriteLine("Nested!");
                        }
                    }
                }
            }
        }
    }
}
