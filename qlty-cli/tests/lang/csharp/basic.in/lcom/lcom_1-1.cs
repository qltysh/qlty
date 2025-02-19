// lcom=1

class Klass
{
    public Klass()
    {
    }

    public string Foo()
    {
        return this.Baz();
    }

    private string Baz()
    {
        return "baz method called";
    }
}
