// lcom=1

class Klass
{
    private string bar;

    public Klass()
    {
    }

    public string GetBar()
    {
        return this.bar;
    }

    public string Foo()
    {
        return this.GetBar();
    }
}