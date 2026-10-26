Public Class BooleanLogic
    Private foo As Integer
    Private bar As Integer

    Private Sub F0()
        Dim x = foo - bar + 1
    End Sub
End Class

Public Class BooleanLogic1
    Private foo As Boolean
    Private bar As Boolean
    Private baz As Boolean
    Private qux As Boolean
    Private zoo As Boolean

    Private Sub F1()
        If foo AndAlso bar AndAlso baz AndAlso qux AndAlso zoo Then
            Return
        End If
    End Sub
End Class
