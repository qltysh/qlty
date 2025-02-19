using System;

public class Identical
{
    public static double[] f0(double[] numbers)
    {
        double sum = 0;
        foreach (double num in numbers)
        {
            sum += num;
        }
        double mean = sum / numbers.Length;

        double[] sortedNumbers = (double[])numbers.Clone();
        Array.Sort(sortedNumbers);

        double median;
        int length = sortedNumbers.Length;
        if (length % 2 == 0)
        {
            median = (sortedNumbers[length / 2 - 1] + sortedNumbers[length / 2]) / 2.0;
        }
        else
        {
            median = sortedNumbers[length / 2];
        }

        return new double[] { mean, median };
    }

    public static double[] f1(double[] numbers)
    {
        double sum = 0;
        foreach (double num in numbers)
        {
            sum += num;
        }
        double mean = sum / numbers.Length;

        double[] sortedNumbers = (double[])numbers.Clone();
        Array.Sort(sortedNumbers);

        double median;
        int length = sortedNumbers.Length;
        if (length % 2 == 0)
        {
            median = (sortedNumbers[length / 2 - 1] + sortedNumbers[length / 2]) / 2.0;
        }
        else
        {
            median = sortedNumbers[length / 2];
        }

        return new double[] { mean, median };
    }
}
