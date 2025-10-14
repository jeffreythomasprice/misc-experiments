case class MatchResult[T](
    result: T,
    remainder: String
)

type Matcher[T] = String => Option[MatchResult[T]]

def stringLiteral(s: String, caseSensitive: Boolean = true): Matcher[String] =
    input =>
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

extension (s: String)
    def toMatcher: Matcher[String] =
        stringLiteral(s)

def charRange(lower: Char, upper: Char): Matcher[Char] =
    input =>
        input.headOption.match
            case Some(value) if value >= lower && value <= upper =>
                Some(
                  MatchResult(result = value, remainder = input.substring(1))
                )
            case _ => None

def list[T](m: Matcher[T]*): Matcher[List[T]] =
    m.toList.match
        case head :: next =>
            val matchTail = list(next*)
            input =>
                head(input).match
                    case Some(MatchResult(headResult, remainder)) =>
                        matchTail(remainder).match
                            case Some(MatchResult(tailResult, remainder)) =>
                                Some(MatchResult(result = headResult :: tailResult, remainder))
                            case None =>
                                None
                    case None =>
                        None
        case Nil =>
            input => Some(MatchResult(result = List(), remainder = input))

extension [T](l: List[Matcher[T]])
    def toListMatcher: Matcher[List[T]] =
        list(l*)

def tuple2[T1, T2](m1: Matcher[T1], m2: Matcher[T2]): Matcher[(T1, T2)] =
    input =>
        m1(input).match
            case Some(MatchResult(r1, remainder)) =>
                m2(remainder).match
                    case Some(MatchResult(r2, remainder)) =>
                        Some(MatchResult((r1, r2), remainder))
                    case None => None
            case None => None

extension [T1, T2](m: (Matcher[T1], Matcher[T2]))
    def toMatcher: Matcher[(T1, T2)] =
        val (m1, m2) = m
        tuple2(m1, m2)

def tuple3[T1, T2, T3](m1: Matcher[T1], m2: Matcher[T2], m3: Matcher[T3]): Matcher[(T1, T2, T3)] =
    input =>
        m1(input).match
            case Some(MatchResult(r1, remainder)) =>
                m2(remainder).match
                    case Some(MatchResult(r2, remainder)) =>
                        m3(remainder).match
                            case Some(MatchResult(r3, remainder)) =>
                                Some(MatchResult((r1, r2, r3), remainder))
                            case None => None
                    case None => None
            case None => None

extension [T1, T2, T3](m: (Matcher[T1], Matcher[T2], Matcher[T3]))
    def toMatcher: Matcher[(T1, T2, T3)] =
        val (m1, m2, m3) = m
        tuple3(m1, m2, m3)

/*
TODO anyOf
def anyOf[T](m: Matcher[T]*): Matcher[List[T]] =

TODO optional
TODO atLeastZero
TODO atLeastOne
 */
