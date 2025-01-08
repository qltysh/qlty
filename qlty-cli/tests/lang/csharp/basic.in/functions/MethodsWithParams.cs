public class MethodsWithParams
{
    private string bar;
    private string baz;

    public MethodsWithParams()
    {
        this.bar = "";
        this.baz = "";
    }

    public string DoSomething(string baz, string bar)
    {
        this.bar = bar;
        this.baz = baz;
        return this.bar + this.baz;
    }
}