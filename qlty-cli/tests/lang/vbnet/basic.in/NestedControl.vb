Imports System

Public Class Main
    Public Shared Sub NotNested(foo As String, bar As String)
        If (foo = "cat" AndAlso bar = "dog") OrElse (foo = "dog" AndAlso bar = "cat") Then
            Console.WriteLine("Got a cat and a dog!")
        Else
            Console.WriteLine("Got nothing")
        End If
    End Sub

    Public Shared Sub F0(bar As Boolean, baz As Boolean, qux As Boolean, quux As Boolean)
        If bar Then
            If baz Then
                If qux Then
                    If quux Then
                        Console.WriteLine("Not deeply nested enough!")
                    End If
                End If
            End If
        End If
    End Sub

    Public Shared Function F2(foo As Integer) As String
        Select Case foo
            Case 1
                Return "bar1"
            Case 2
                Return "bar2"
            Case 3
                Return "bar3"
            Case 4
                Return "bar4"
            Case 5
                Return "bar5"
            Case 6
                Return "bar6"
            Case 7
                Return "bar7"
            Case 8
                Return "bar8"
            Case 9
                Return "bar9"
            Case 10
                Return "bar10"
            Case Else
                Throw New ArgumentException("Invalid foo value")
        End Select
    End Function
End Class
