namespace Fields
{
    public class GetterSetterDeclaration
    {
        public string Field { get; set; }
        public string _field2

        public static void Main(string[] args)
        {
            GetterSetterDeclaration obj = new GetterSetterDeclaration();
            obj.Field = "Hello";
            obj._field2 = "World";
            System.Console.WriteLine(obj.Field);
        }
    }
}