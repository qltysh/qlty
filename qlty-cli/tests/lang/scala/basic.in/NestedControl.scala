class NestedControl {
  def go(xs: Seq[Int]): Unit = {
    if (xs.nonEmpty) {
      for (x <- xs) {
        if (x > 0) {
          for (y <- xs) {
            if (y > x) {
              println(y)
            }
          }
        }
      }
    }
  }
}
