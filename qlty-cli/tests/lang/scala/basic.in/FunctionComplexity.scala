class FunctionComplexity {
  def complex(a: Int, b: Int, c: Int): Int = {
    if (a > 0) {
      if (b > 0) {
        if (c > 0) {
          if (a > b) {
            if (b > c) {
              if (a + b > 10) {
                if (a * b > 100) {
                  return a
                }
              }
            }
          }
        }
      }
    }
    if (a < 0) {
      if (b < 0) {
        return b
      }
    }
    0
  }
}
