class Identical {
  def computeMetric(a: Int, b: Int, c: Int): Int = {
    val total = a + b + c
    if (total > 100) {
      total * 2
    } else if (total > 50) {
      total + 25
    } else {
      total
    }
  }

  def computeMetricCopy(a: Int, b: Int, c: Int): Int = {
    val total = a + b + c
    if (total > 100) {
      total * 2
    } else if (total > 50) {
      total + 25
    } else {
      total
    }
  }
}
