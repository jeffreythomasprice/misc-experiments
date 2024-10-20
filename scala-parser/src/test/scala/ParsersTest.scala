import org.scalatest.funsuite.AnyFunSuite
import scala.util.Success
import scala.util.Failure
import org.scalatest.TryValues._
import org.scalatest.TryValues
import org.scalatest.matchers.should.Matchers
import parsers.*

class ParsersTest extends AnyFunSuite with Matchers with TryValues {
  test("string, success") {
    var parser = string("foo")
    var result = parser("foobar")
    result.success.value shouldBe ParseResult("foo", "bar")
  }

  test("string, failure") {
    var parser = string("bar")
    var result = parser("foobar")
    result.failure.exception shouldBe a[IllegalArgumentException]
    result.failure.exception.getMessage() shouldBe "expected bar"
  }

  test("regex, success") {
    var parser = parsers.regex("""\d+""".r)
    var result = parser("123abc")
    result.success.value shouldBe ParseResult("123", "abc")
  }

  test("regex, failure") {
    var parser = parsers.regex("""\d+""".r)
    var result = parser("abc123")
    result.failure.exception shouldBe a[IllegalArgumentException]
    result.failure.exception.getMessage() shouldBe "expected \\d+"
  }

  test("map, success") {
    var parser = string("foo")
      .map(result => Success(result.length))
    var result = parser("foobar")
    result.success.value shouldBe ParseResult(3, "bar")
  }

  test("map, the original parser fails") {
    var parser = string("bar")
      .map(result => Success(result.length))
    var result = parser("foobar")
    result.failure.exception shouldBe a[IllegalArgumentException]
    result.failure.exception.getMessage() shouldBe "expected bar"
  }

  test("map, the map function fails") {
    var parser = string("foo")
      .map(result => Failure(Exception("baz")))
    var result = parser("foobar")
    result.failure.exception.getMessage() shouldBe "baz"
  }

  test("seq2, success") {
    var parser = seq2(string("1"), string("22"))
    var result = parser("122asdf")
    result.success.value shouldBe ParseResult(("1", "22"), "asdf")
  }

  test("seq2, failed on missing first") {
    var parser = seq2(string("1"), string("22"))
    var result = parser("*22asdf")
    result.failure.exception shouldBe a[IllegalArgumentException]
    result.failure.exception.getMessage() shouldBe "expected 1"
  }

  test("seq2, failed on missing second") {
    var parser = seq2(string("1"), string("22"))
    var result = parser("1*2asdf")
    result.failure.exception shouldBe a[IllegalArgumentException]
    result.failure.exception.getMessage() shouldBe "expected 22"
  }

  test("seq3, success") {
    var parser = seq3(string("1"), string("22"), string("333"))
    var result = parser("122333asdf")
    result.success.value shouldBe ParseResult(("1", "22", "333"), "asdf")
  }

  test("seq3, failed on missing first") {
    var parser = seq3(string("1"), string("22"), string("333"))
    var result = parser("*22333asdf")
    result.failure.exception shouldBe a[IllegalArgumentException]
    result.failure.exception.getMessage() shouldBe "expected 1"
  }

  test("seq3, failed on missing second") {
    var parser = seq3(string("1"), string("22"), string("333"))
    var result = parser("1*2333asdf")
    result.failure.exception shouldBe a[IllegalArgumentException]
    result.failure.exception.getMessage() shouldBe "expected 22"
  }

  test("seq3, failed on missing third") {
    var parser = seq3(string("1"), string("22"), string("333"))
    var result = parser("122*33asdf")
    result.failure.exception shouldBe a[IllegalArgumentException]
    result.failure.exception.getMessage() shouldBe "expected 333"
  }

  test("any, success on first") {
    var parser = any(
      string("1"),
      string("2")
    )
    var result = parser("1asdf");
    result.success.value shouldBe ParseResult("1", "asdf")
  }

  test("any, success on second") {
    var parser = any(
      string("1"),
      string("2")
    )
    var result = parser("2asdf");
    result.success.value shouldBe ParseResult("2", "asdf")
  }

  test("any, failure") {
    var parser = any(
      string("1"),
      string("2")
    )
    var result = parser("asdf");
    result.failure.exception shouldBe a[ExpectedOneOfException]
    val exceptions = result.failure.exception
      .asInstanceOf[ExpectedOneOfException]
      .exceptions
    exceptions.length shouldBe 2
    exceptions(0) shouldBe a[IllegalArgumentException]
    exceptions(0).getMessage() shouldBe "expected 1"
    exceptions(1) shouldBe a[IllegalArgumentException]
    exceptions(1).getMessage() shouldBe "expected 2"
  }

  test("skip, success") {
    var parser = skip(string("foo"), string("bar"))
    var result = parser("foobarbaz");
    result.success.value shouldBe ParseResult("bar", "baz")
  }

  test("skip, failure") {
    var parser = skip(string("foo"), string("bar"))
    var result = parser("barbaz");
    result.failure.exception shouldBe a[IllegalArgumentException]
    result.failure.exception.getMessage() shouldBe "expected foo"
  }

  test("bracketed, success") {
    var parser = bracketed(string("("), string("foo"), string(")"))
    var result = parser("(foo)bar")
    result.success.value shouldBe ParseResult("foo", "bar")
  }

  test("bracketed, failure") {
    var parser = bracketed(string("("), string("foo"), string(")"))
    var result = parser("foo)bar")
    result.failure.exception shouldBe a[IllegalArgumentException]
    result.failure.exception.getMessage() shouldBe "expected ("
  }

  /*
  TODO test repeat, success, unbounded, no matches
  TODO test repeat, success, unbounded, some matches
  TODO test repeat, failure, bounded on low end, not enough matches
  TODO test repeat, success, bounded on low end
  TODO test repeat, success, bounded on high end, no matches
  TODO test repeat, success, bounded on high end, some matches
  TODO test repeat, failure, bounded on high end, too many matches
  TODO test repeat, failure, bounded on both ends, not enough matches
  TODO test repeat, success, bounded on both ends, just enough matches
  TODO test repeat, failure, bounded on both ends, too many matches
  TODO test repeat, success, bounded on both ends, step=2, actual number is even
  TODO test repeat, success, bounded on both ends, step=2, actual number is odd so there's a leftover
   */
}
