Imports System

Public Class FileComplexity
    Public Shared Sub Main(args As String())
        Dim foo As Integer = 42

        If foo > 0 Then
            If foo > 10 Then
                If foo < 20 Then
                    If foo Mod 2 = 0 Then
                        If foo Mod 3 = 0 Then
                            If foo Mod 5 = 0 Then
                                If foo Mod 7 = 0 Then
                                    If foo Mod 11 = 0 Then
                                        If foo Mod 13 = 0 Then
                                            If foo Mod 17 = 0 Then
                                                Console.WriteLine("Nested!")
                                            End If
                                        End If
                                    End If
                                End If
                            End If
                        End If
                    End If
                End If
            End If
        End If
    End Sub
End Class
