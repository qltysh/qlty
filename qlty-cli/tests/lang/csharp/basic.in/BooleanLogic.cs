public class BooleanLogic
{
    private int foo;
    private int bar;

    private void F0()
    {
        var x = foo - bar + 1;
    }
}

public class BooleanLogic1
{
    private bool foo;
    private bool bar;
    private bool baz;
    private bool qux;
    private bool zoo;

    private void F1()
    {
        if (foo && bar && baz && qux && zoo)
        {
            return;
        }
    }
}

