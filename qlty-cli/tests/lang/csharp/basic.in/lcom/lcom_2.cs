// lcom=2

class KlassA
{
    private string aaa;
    private string bbb;

    public string GetBbb()
    {
        return this.bbb;
    }

    public string GetAaa()
    {
        return this.aaa;
    }

    private string Foo()
    {
        return this.GetAaa();
    }

    private string Bar()
    {
        return this.GetBbb();
    }
}

class KlassB
{
    private string baz;

    public string GetBar()
    {
        return this.baz;
    }

    private string Foo()
    {
        return this.GetBar();
    }

    private string Bar()
    {
        return this.GetBar();
    }
}