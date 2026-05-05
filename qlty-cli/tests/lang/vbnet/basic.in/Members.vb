Public Class MixedCaseMembers
    Private _value As Integer

    Public Sub New()
        Me.Value = 0
    End Sub

    Public Sub SetValue(v As Integer)
        Me.Value = v
    End Sub

    Public Function GetValue() As Integer
        Return me.value
    End Function

    Public Sub DoWork()
        Me.SetValue(42)
    End Sub

    Public Property Value As Integer
        Get
            Return _value
        End Get
        Set(value As Integer)
            _value = value
        End Set
    End Property
End Class

Public Class Disconnected
    Public Sub MethodA()
        Me.Alpha()
    End Sub

    Public Sub MethodB()
        Me.Beta()
    End Sub

    Public Sub Alpha()
    End Sub

    Public Sub Beta()
    End Sub
End Class
