class StringsSuite extends munit.FunSuite {
    test("dedent") {
        val input = " foo\n  bar\n   baz"
        val actual = dedent(input)
        val expected = "foo\n bar\n  baz"
        assertEquals(actual, expected)
    }
}
