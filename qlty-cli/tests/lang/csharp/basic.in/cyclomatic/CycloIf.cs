namespace Cyclomatic
{
  class CycloIf {
    public static string Main(string args[]) {
      int x = 1;
      if (x > 0) {
        int y = 1;
      }
    }
  }
}