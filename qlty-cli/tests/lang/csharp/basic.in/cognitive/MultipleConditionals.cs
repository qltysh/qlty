namespace cognitive
{
  class MultipleConditionals {
    public static string Main(int foo) {
      if (foo >= 80 && foo <= 100) {
        return "Most complex!";
      } else if (foo >= 60 && foo <= 79) {
        return "Very complex";
      } else if (foo >= 40 && foo <= 59) {
        return "Somewhat complex";
      } else if (foo >= 20 && foo <= 39) {
        return "Not complex";
      } else if (foo >= 0 && foo <= 19) {
        return "Least complex!";
      } else {
        return null;
      }
    }
  }
}