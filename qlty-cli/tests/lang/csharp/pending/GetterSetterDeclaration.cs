namespace Fields
{
    public class GetterSetterDeclaration
    {
        public string Field { get; set; }

        public static void Main(string[] args)
        {
            GetterSetterDeclaration obj = new GetterSetterDeclaration();
            obj.Field = "Hello";
            System.Console.WriteLine(obj.Field);
        }
    }
}