Imports System

Public Class Returns
    Public Sub F0()
    End Sub

    Public Sub F1()
        Return
    End Sub

    Public Sub F2()
        If True Then
            Return
        Else
            Return
        End If
    End Sub

    Public Sub F3()
        If True Then
            Return
        ElseIf True Then
            Return
        Else
            Return
        End If
    End Sub

    Public Sub F4()
        If True Then
            Return
        ElseIf True Then
            Return
        ElseIf True Then
            Return
        Else
            Return
        End If
    End Sub
End Class
