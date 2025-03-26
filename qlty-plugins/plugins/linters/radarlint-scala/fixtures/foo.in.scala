import java.sql.{Connection, DriverManager, Statement}

object SecurityIssues {

  def insecureDatabaseConnection(): Unit = {
    // 🚨 S3649: Hardcoded credentials (Security issue)
    val dbUser = "admin"
    val dbPassword = "password123"

    // 🚨 S2077: SQL Injection vulnerability (String concatenation in SQL)
    val userInput = "Robert'); DROP TABLE users; --"
    val query = s"SELECT * FROM users WHERE name = '$userInput'"

    val connection: Connection = DriverManager.getConnection(
      "jdbc:mysql://localhost:3306/mydb", dbUser, dbPassword
    )
    val statement: Statement = connection.createStatement()
    val resultSet = statement.executeQuery(query)

    while (resultSet.next()) {
      println(resultSet.getString("username"))
    }
  }

  def inefficientLoop(): Unit = {
    val numbers = List(1, 2, 3, 4, 5)

    // 🚨 S4034: Inefficient loop (should use `foreach`)
    for (i <- 0 until numbers.length) {
      println(numbers(i))
    }
  }

  def emptyCatchBlock(): Unit = {
    try {
      val result = 10 / 0
    } catch {
      case e: ArithmeticException => // 🚨 S2221: Empty catch block (ignoring the exception)
    }
  }

  def main(args: Array[String]): Unit = {
    insecureDatabaseConnection()
    inefficientLoop()
    emptyCatchBlock()
  }
}
