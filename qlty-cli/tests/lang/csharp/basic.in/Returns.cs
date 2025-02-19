using System;

public class Returns
{
    public void F0()
    {
    }

    public void F1()
    {
        return;
    }

    public void F2()
    {
        if (true)
        {
            return;
        }
        else
        {
            return;
        }
    }

    public void F3()
    {
        if (true)
        {
            return;
        }
        else if (true)
        {
            return;
        }
        else
        {
            return;
        }
    }

    public void F4()
    {
        if (true)
        {
            return;
        }
        else if (true)
        {
            return;
        }
        else if (true)
        {
            return;
        }
        else
        {
            return;
        }
    }
}
