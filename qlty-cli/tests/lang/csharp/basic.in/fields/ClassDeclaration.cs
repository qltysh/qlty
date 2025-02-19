namespace Fields
{
    class Foo
    {
        public string bar;
        public string baz;

        // Constructor
        public Foo()
        {
            this.bar = "";
            this.baz = "";
        }
    }

    public class ClassDeclaration
    {
        public static void Main(string[] args)
        {
            System.Console.WriteLine(DoSomething());
        }

        public static string DoSomething()
        {
            Foo foo = new Foo();
            foo.bar = "Hello";
            foo.baz = "World";
            return foo.bar + foo.baz;
        }
    }
}