namespace Fields
{
    public class StaticFieldDeclaration
    {
        public static string staticField = "staticValue";

        public static void Main(string[] args)
        {
            System.Console.WriteLine(StaticFieldDeclaration.staticField);
        }
    }
}