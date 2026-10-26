Imports System
Imports System.IO

Public Class Main
    Public Shared Sub F1()
        Try
            Dim tempFilePath As String = Path.Combine(
                Path.GetTempPath(),
                $"ruby{Guid.NewGuid()}.kt"
            )

            Using writer As New StreamWriter(tempFilePath)
                writer.Write("foo(...args)")
            End Using

            Dim tree As Object = ParseFile(tempFilePath)

            Bar()
        Catch e As IOException
            Console.Error.WriteLine(e)
        End Try
    End Sub

    Public Shared Function ParseFile(filePath As String) As Object
        Return New Object()
    End Function

    ' Foo
    Public Shared Sub F2()
        Bar() ' does not count as comment line
    End Sub

    ' multi-line comment
    ' line1
    ' line2
    ' line4

    Public Shared Sub F3()
        Bar()
    End Sub

    Public Shared Sub Bar()
        Console.WriteLine("bar() called")
    End Sub
End Class
