case class MatchResult[T](
    result: T,
    remainder: String
)

type Matcher[T] = String => Option[MatchResult[T]]

def stringLiteral(s: String, caseSensitive: Boolean = true): Matcher[String] =
    (input: String) =>
        val hasPrefix =
            if caseSensitive then input.startsWith(s)
            else input.toLowerCase().startsWith(s.toLowerCase())
        if hasPrefix then
            Some(
              MatchResult(
                result = input.substring(0, s.length()),
                remainder = input.substring(s.length())
              )
            )
        else None

def charRange(lower: Char, upper: Char): Matcher[Char] =
    (input: String) =>
        input.headOption.match
            case Some(value) if value >= lower && value <= upper =>
                Some(
                  MatchResult(result = value, remainder = input.substring(1))
                )
            case _ => None

/*
TODO seqList
TODO seq2
TODO seq3
TODO anyOf
TODO optional
TODO atLeastZero
TODO atLeastOne
 */
