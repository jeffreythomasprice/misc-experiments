import calculator
import gleeunit/should
import parser

pub fn skip_whitespace_test() {
  let p = calculator.skip_whitespace(parser.string("foo"))
  p("") |> should.equal(Error(Nil))
  p("   ") |> should.equal(Error(Nil))
  p("foo") |> should.equal(Ok(#("", "foo")))
  p("   foo") |> should.equal(Ok(#("", "foo")))
  p(" \t\r\nfoo   ") |> should.equal(Ok(#("   ", "foo")))
}

pub fn number_test() {
  let p = calculator.number()
  p("12345") |> should.equal(Ok(#("", 12_345.0)))
  //   TODO real number test
}
