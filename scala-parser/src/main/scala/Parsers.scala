package parsers

import scala.annotation.tailrec
import scala.util.matching.Regex
import scala.util.{Failure, Success, Try}

case class ParseResult[T](
    result: T,
    remainder: String
)

type Parser[T] = String => Try[ParseResult[T]]

def string(s: String): Parser[String] =
  input =>
    if input.startsWith(s) then
      Success(ParseResult(s, input.substring(s.length)))
    else Failure(IllegalArgumentException(s"expected $s"))

def regex(r: Regex): Parser[String] =
  input =>
    r.findPrefixMatchOf(input) match
      case Some(m) => Success(ParseResult(m.matched, input.substring(m.end)))
      case None    => Failure(IllegalArgumentException(s"expected $r"))

extension [T, R](p: Parser[T])
  def map(f: T => Try[R]): Parser[R] =
    input =>
      for
        ParseResult(intermediate, remainder) <- p(input)
        result <- f(intermediate)
      yield ParseResult(result, remainder)

def seq2[T1, T2](p1: Parser[T1], p2: Parser[T2]): Parser[(T1, T2)] =
  input =>
    for
      ParseResult(r1, remainder) <- p1(input)
      ParseResult(r2, remainder) <- p2(remainder)
    yield ParseResult((r1, r2), remainder)

def seq3[T1, T2, T3](
    p1: Parser[T1],
    p2: Parser[T2],
    p3: Parser[T3]
): Parser[(T1, T2, T3)] =
  input =>
    for
      ParseResult(r1, remainder) <- p1(input)
      ParseResult(r2, remainder) <- p2(remainder)
      ParseResult(r3, remainder) <- p3(remainder)
    yield ParseResult((r1, r2, r3), remainder)

class ExpectedOneOfException(val exceptions: List[Throwable]) extends Exception

def any[T](parsers: Parser[T]*): Parser[T] =
  input =>
    val results = parsers.map(_(input))
    val (successes, failures) = parsers
      .map(_(input))
      .partitionMap(_ match
        case Success(value)     => Left(value)
        case Failure(exception) => Right(exception)
      )
    (successes.headOption, failures) match
      case (Some(result), _) => Success(result)
      case (_, failures)     => Failure(ExpectedOneOfException(failures.toList))

def skip[T1, T2](p1: Parser[T1], p2: Parser[T2]): Parser[T2] =
  input =>
    for ParseResult((_, result), remainder) <- seq2(p1, p2)(input)
    yield ParseResult(result, remainder)

def bracketed[T1, T2, T3](
    p1: Parser[T1],
    p2: Parser[T2],
    p3: Parser[T3]
): Parser[T2] =
  input =>
    for ParseResult((_, result, _), remainder) <- seq3(p1, p2, p3)(input)
    yield ParseResult(result, remainder)

enum RangeOptions {
  case Unbounded
  case Bounded(value: Int)
}

case class Range(val lower: RangeOptions, val upper: RangeOptions) {
  def contains(x: Int): Boolean =
    (lower, upper) match
      case (RangeOptions.Unbounded, RangeOptions.Unbounded)      => true
      case (RangeOptions.Bounded(lower), RangeOptions.Unbounded) => x >= lower
      case (RangeOptions.Unbounded, RangeOptions.Bounded(upper)) => x <= upper
      case (RangeOptions.Bounded(lower), RangeOptions.Bounded(upper)) =>
        x >= lower && x <= upper
}

def repeat[T](p: Parser[T], r: Range): Parser[List[T]] =
  /*
	recursively match the next element and update the partial results as we go
	returns either a list of results that satisfy the range, or none
   */
  @tailrec
  def helper(
      input: String,
      allResultsSoFar: List[T],
      lastSuccessfulResults: Option[List[T]]
  ): Try[ParseResult[List[T]]] =
    // if adding one more result to total results would put us past the end of the range then we're done
    if r.contains(allResultsSoFar.length) && !r.contains(
        allResultsSoFar.length + 1
      )
    then
      lastSuccessfulResults match
        // we had some previous success
        case Some(x) => Success(ParseResult(x, input))
        // we never had a success so just try to be descriptive for what we were looking for
        case None =>
          Failure(
            IllegalArgumentException(
              s"didn't get enough matches for $p, valid range $r"
            )
          )
    else
      // see if we can actually make another match
      p(input) match
        case Success(ParseResult(newResult, remainder)) =>
          // we have a new result
          val newAllResults = newResult :: allResultsSoFar
          // if that new list works then keep it, otherwise whatever we last had as the successful list
          // e.g. we might be skipping some because our range has an increment other than 1
          val newLastSuccessfulResults =
            if r.contains(newAllResults.length) then Some(newAllResults)
            else lastSuccessfulResults
          // recurse
          helper(remainder, newAllResults, newLastSuccessfulResults)
        case Failure(e) =>
          // failed to match again, just return whatever we have
          lastSuccessfulResults match
            // we had some previous success
            case Some(x) => Success(ParseResult(x, input))
            // we never had a success so just try to be descriptive for what we were looking for
            case None =>
              Failure(
                java.lang.IllegalArgumentException(
                  s"not enough results, was looking for $r, but didn't get enough matches"
                )
              )

  // if the range allows for an empty result as a success then we can start with that
  val initialSuccessResults = if r.contains(0) then Some(List()) else None
  input => helper(input, List(), initialSuccessResults)

// TODO optional
