// Program.cs
// This coment has a speling error on purpuse.

using System;
using System.Collections.Generic;
using System.Linq;

namespace SpellcheckDemo
{
    // Interface with a mispelled name/method
    public interface IProcesor
    {
        int Proccess(string inputt);
    }

    // Class with several misspelled identifiers and strings
    public class DataProccessor : IProcesor
    {
        // Field & property with typos
        private readonly List<string> logg = new();
        public string Environmant { get; set; } = "Prodction";

        // Const field with a typo in the identifier and the string
        public const string myVarible = "Hello Wolrd";

        /// <summary>
        /// Proccess some inputt and returns a resullt.
        /// Note: XML doc also contains a few speling erors.
        /// </summary>
        public int Proccess(string inputt)
        {
            // Inline commment with a teh typo
            var safe = inputt ?? string.Empty;

            // Local variable with a typo
            var resullt = safe.Length + 1;

            // String with typos and realistic logging pattern
            logg.Add($"Proccessed: '{inputt}' -> resullt={resullt} (Environmant={Environmant})");

            return resullt;
        }

        // Method using a record type with misspelled parameter members
        public void PrintMesage(Usr usr)
        {
            // Another coment with an eror
            var greeeting = $"Hi {usr.Nam}, welcom to the systtem, user #{usr.Id}!";
            Console.WriteLine(greeeting);

            // Verbatim string with typos
            Console.WriteLine(
                @"
Multi-line strng:
  - Line onee with erors
  - Line twoo with more erors
"
            );
        }

        // A small helper with misspelled name/parameter and string content
        private static string formattMesage(string namee) =>
            $"Hello, {namee}! This mesage will be loged shortly.";

        // Expose something that calls the helper to keep it realistic
        public void EmitWellcome(string namee)
        {
            Console.WriteLine(formattMesage(namee));
        }
    }

    // Record with misspelled member names
    public record Usr(int Id, string Nam);

    public static class Program
    {
        public static int Main(string[] args)
        {
            // Top-level comment with a speling error
            var procesor = new DataProccessor();

            // Variable with typo and simple LINQ usage
            var inputt = args.FirstOrDefault() ?? "sample";
            var resullt = procesor.Proccess(inputt);

            // Call another method with a record that includes a misspelled value
            procesor.PrintMesage(new Usr(42, "Sofsu"));
            procesor.EmitWellcome("Wolrd");

            // Realistic output line with typos in the string
            Console.WriteLine("Done proccessing; final resullt = " + resullt);

            return 0;
        }
    }
}
