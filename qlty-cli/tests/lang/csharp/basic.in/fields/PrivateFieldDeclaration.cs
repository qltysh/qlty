namespace Fields
{
    public class PrivateFieldDeclaration
    {
        private string privateField = "privateValue";

        public string GetPrivateField()
        {
            return this.privateField;
        }

        public static void Main(string[] args)
        {
            // Entry point
        }
    }
}