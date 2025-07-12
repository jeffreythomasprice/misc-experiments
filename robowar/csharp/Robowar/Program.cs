using Robowar;

var input = new Input("Hello, World!");
Console.WriteLine(input);
Console.WriteLine(new LiteralParser("Hello").TryParse(input));

