def dedent(s: String): String =
    def allElementsEqual[T](inputs: List[T]): Boolean =
        inputs.match
            case a :: b :: next => a == b && allElementsEqual(b :: next)
            case _ :: Nil       => true
            case Nil            => true

    def longestCommonWhitespace(inputs: List[String]): String =
        val headChars =
            inputs.map(x => x.headOption)
        (inputs, allElementsEqual(headChars), headChars.head).match
                // no inputs
                case (Nil, _, _) => ""
                // first char isn't the same in all inputs
                case (_, false, _) => ""
                // first char is the same whitespace in all inputs
                case (_, true, Some(head)) if head.isWhitespace =>
                    head.toString() + longestCommonWhitespace(
                      inputs.map(x => x.stripPrefix(head.toString()))
                    )
                    // first char isn't whitespace
                case (_, true, _) => ""

    val lines =
        s.split("\n").toList.map(x => if x.trim().isEmpty() then "" else x)
    val prefix = longestCommonWhitespace(lines.filter(x => !x.isEmpty()))
    lines.map(x => x.stripPrefix(prefix)).mkString("\n")
