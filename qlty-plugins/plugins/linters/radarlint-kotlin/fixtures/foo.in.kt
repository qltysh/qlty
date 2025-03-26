import java.sql.Connection
import java.sql.DriverManager
import java.sql.Statement

fun main() {
    // TODO: This is a test
    val issues = SecurityIssues()
    issues.insecureDatabaseConnection()
    issues.inefficientLoop()
    issues.emptyCatchBlock()
}
