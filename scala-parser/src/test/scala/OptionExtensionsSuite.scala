class OptionExtensionsSuite extends munit.FunSuite:
    test("defined") {
        var x = false
        assertEquals(Some(42).ifDefined((_) => x = true), Some(42))
        assertEquals(x, true)

        x = false
        assertEquals(None.ifDefined((_) => x = true), None)
        assertEquals(x, false)
    }

    test("empty") {
        var x = false
        assertEquals(Some(42).ifEmpty(() => x = true), Some(42))
        assertEquals(x, false)

        x = false
        assertEquals(None.ifEmpty(() => x = true), None)
        assertEquals(x, true)
    }
