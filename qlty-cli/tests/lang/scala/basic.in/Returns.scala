class Returns {
  def manyReturns(a: Int): Int = {
    if (a < 0) return -1
    if (a == 0) return 0
    if (a < 10) return 1
    if (a < 100) return 2
    if (a < 1000) return 3
    if (a < 10000) return 4
    return 5
  }
}
