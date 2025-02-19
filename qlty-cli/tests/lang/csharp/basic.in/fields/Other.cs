namespace Fields
{
    public class Other
    {
        public int foo;
        public int bar;

        public static void Main(string[] args)
        {
            Other other = new Other();
            other.foo = 1;
            int barValue = other.bar;
        }
    }
}