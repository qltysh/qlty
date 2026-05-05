class ForComprehension {
  def collect(xs: Seq[Int], ys: Seq[Int]): Seq[Int] = {
    for {
      x <- xs
      y <- ys if y > 0
    } yield x + y
  }
}
