namespace Cyclomatic
{
  class CycloIfElse {
    public static void Main(string[] args) {
      int x = 1;
      if (x > 0) {
        int y = 1;
      } else {
        int y = 2;
      }
    }
  }
}