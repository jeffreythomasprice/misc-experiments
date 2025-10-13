enum Foo:
    case Bar(x: String)
    case Baz()

@main def hello(): Unit =
    println("Hello world!")
    println(msg)
    println(Foo.Baz())
    println(Foo.Bar("test"))

def msg = "I was compiled by Scala 3. :)"
