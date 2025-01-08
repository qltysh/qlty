// lcom = 0 for all the tests in this file, totalling 0

class Foo
{
    private string bar;

    public string foo()
    {
        return this.bar;
    }
}

class Klass1
{
    public Klass1()
    {
    }

    public object foo()
    {
        return null;
    }
}

class Klass2
{
    private Bar bar;

    public Klass2(Bar bar)
    {
        this.bar = bar;
    }

    public object foo()
    {
        return this.bar.getBaz();
    }
}

class Bar
{
    private string baz;

    public string getBaz()
    {
        return this.baz;
    }
}