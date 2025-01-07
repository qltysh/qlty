namespace Cyclomatic
{
  class CycloIfElseIfElse {
    public static void Main(string[] args) {
      int x = 1;
      if (x > 0) {
        int y = 1;
      } else if (x < 0) {
        int y = 2;
      } else {
        int y = 3;
      }
    }
  }
}