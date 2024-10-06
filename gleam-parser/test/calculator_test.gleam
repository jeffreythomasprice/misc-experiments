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
  let p = calculator.number
  p("12345") |> should.equal(Ok(#("", 12_345.0)))
  p("12.345") |> should.equal(Ok(#("", 12.345)))
  p("-1") |> should.equal(Ok(#("", -1.0)))
  p("2e5") |> should.equal(Ok(#("", 2.0e5)))
  p("3.7e-6") |> should.equal(Ok(#("", 3.7e-6)))
}

pub fn expression_test() {
  let p = calculator.expression

  let value = p("1 + 2")
  value
  |> should.equal(
    Ok(#("", calculator.Add(calculator.Number(1.0), calculator.Number(2.0)))),
  )
  let assert Ok(#(_, value)) = value
  calculator.eval(value) |> should.equal(3.0)
  // TODO more expression tests
}
