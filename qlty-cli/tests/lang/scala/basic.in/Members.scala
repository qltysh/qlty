class Members(val name: String) {
  val id: Int = 1
  var count: Int = 0
  def greet(): String = s"Hello, $name"
}

object Members {
  def fromString(s: String): Members = new Members(s)
}

trait Greeter {
  def hello(): String
}

case class Point(x: Int, y: Int)
