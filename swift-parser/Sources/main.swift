let parser = string("foo")
let input = "foobar"
print("parser = \(parser)")
print("input = \(input)")
let result = parser.apply(input: input)
print("result = \(result)")
