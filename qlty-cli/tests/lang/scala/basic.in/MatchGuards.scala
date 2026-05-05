class MatchGuards {
  def classify(n: Int): String = n match {
    case x if x < 0 => "negative"
    case 0 => "zero"
    case x if x < 10 => "small"
    case _ => "large"
  }
}
