Imports System

Public Class Parameters
    Public Shared Sub F0()
    End Sub

    Public Shared Sub F1(dog As Object, cat As Object)
    End Sub

    Public Shared Sub F2(a As Object, b As Object, c As Object, d As Object, e As Object, f As Object)
    End Sub

    Public Shared Sub F3()
        Dim foo As Object = Bar(1, 2, 3, 4)
    End Sub

    Public Shared Function Bar(a As Integer, b As Integer, c As Integer, d As Integer) As Object
        Return New Object()
    End Function
End Class
