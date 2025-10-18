@main def hello(): Unit =
    // val s = """
    //     This is a multi-line
    //     string
    //         with some indentation.
    // """
    // println(s"s = \n$s")
    // println(s"dedent(s) = \n${dedent(s)}")
    val m = "aaa".toMatcher.toAtLeastMatcher(0)
    val result = m("aaaaaa___")
    println(s"result = ${result}")
