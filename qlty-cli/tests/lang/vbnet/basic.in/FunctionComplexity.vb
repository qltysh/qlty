Imports System

Public Class FunctionComplexity
    Public Sub Simple()
    End Sub

    Public Sub Complex()
        Dim bar As Integer = 42

        If bar > 0 Then
            If bar > 10 Then
                If bar < 20 Then
                    If bar Mod 2 = 0 Then
                        If bar Mod 3 = 0 Then
                            Console.WriteLine("Nested!")
                        End If
                    End If
                End If
            End If
        End If
    End Sub
End Class
