module Robowar.Tests.Common

open Xunit

let commonTest<'Input, 'Result when 'Result: equality> (f: 'Input -> 'Result) (input: 'Input) (expected: 'Result) =
    let actual = f input
    let message = sprintf "actual != expected\nactual: %A\nexpected: %A" actual expected
    Assert.True(actual.Equals expected, message)
