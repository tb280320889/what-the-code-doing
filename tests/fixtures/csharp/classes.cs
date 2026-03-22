namespace MyApp
{
    public class Calculator
    {
        public int Add(int a, int b)
        {
            return a + b;
        }

        public void Print(string message)
        {
            System.Console.WriteLine(message);
        }
    }

    public struct Point
    {
        public double X;
        public double Y;
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
}
