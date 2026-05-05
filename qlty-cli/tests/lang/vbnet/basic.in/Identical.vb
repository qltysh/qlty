Imports System

Public Class Identical
    Public Shared Function F0(numbers As Double()) As Double()
        Dim sum As Double = 0
        For Each num As Double In numbers
            sum += num
        Next
        Dim mean As Double = sum / numbers.Length

        Dim sortedNumbers As Double() = DirectCast(numbers.Clone(), Double())
        Array.Sort(sortedNumbers)

        Dim median As Double
        Dim length As Integer = sortedNumbers.Length
        If length Mod 2 = 0 Then
            median = (sortedNumbers(length \ 2 - 1) + sortedNumbers(length \ 2)) / 2.0
        Else
            median = sortedNumbers(length \ 2)
        End If

        Return New Double() {mean, median}
    End Function

    Public Shared Function F1(numbers As Double()) As Double()
        Dim sum As Double = 0
        For Each num As Double In numbers
            sum += num
        Next
        Dim mean As Double = sum / numbers.Length

        Dim sortedNumbers As Double() = DirectCast(numbers.Clone(), Double())
        Array.Sort(sortedNumbers)

        Dim median As Double
        Dim length As Integer = sortedNumbers.Length
        If length Mod 2 = 0 Then
            median = (sortedNumbers(length \ 2 - 1) + sortedNumbers(length \ 2)) / 2.0
        Else
            median = sortedNumbers(length \ 2)
        End If

        Return New Double() {mean, median}
    End Function
End Class
