using System;

public class Parameters
{
    public static void F0()
    {
    }

    public static void F1(object dog, object cat)
    {
    }

    public static void F2(object a, object b, object c, object d, object e, object f)
    {
    }

    public static void F3()
    {
        object foo = Bar(1, 2, 3, 4);
    }

    public static object Bar(int a, int b, int c, int d)
    {
        return new object();
    }
}
