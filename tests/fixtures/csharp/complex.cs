using System;
using System.Collections.Generic;
using System.Linq;

namespace MyApp
{
    public class Calculator
    {
        private int _value;

        public Calculator()
        {
            _value = 0;
        }

        public int Add(int a, int b)
        {
            _value = a + b;
            return _value;
        }

        public void Print(string message)
        {
            Console.WriteLine(message);
        }
    }

    public struct Point
    {
        public double X;
        public double Y;

        public Point(double x, double y)
        {
            X = x;
            Y = y;
        }
    }

    public interface IProcessor
    {
        void Process(string data);
    }

    public enum Color
    {
        Red,
        Green,
        Blue
    }

    public static class Extensions
    {
        public static int Square(this int n)
        {
            return n * n;
        }
    }

    class Program
    {
        static void Main(string[] args)
        {
            var calc = new Calculator();
            int result = calc.Add(5, 10);
            Console.WriteLine(result);
        }
    }
}
