using System;

namespace Cognitive
{
    class CountConditionalAssignment
    {
        public static void Main(string[] args)
        {
            int bar = 0;
            bar = bar != 0 ? bar : 10;

            int foo = 0;
            foo = foo != 0 && foo != 10 ? foo : 10;
        }
    }
}
