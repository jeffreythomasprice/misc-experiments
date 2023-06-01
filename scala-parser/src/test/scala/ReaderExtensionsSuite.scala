import java.io.BufferedReader
import java.io.StringReader
import java.nio.CharBuffer

class ReaderExtensionsSuite extends munit.FunSuite:
    test("buffered reader peek") {
        val reader = BufferedReader(StringReader("Hello, World!"))
        val buffer = new Array[Char](100)

        assertEquals(reader.peek(5), Some("Hello"))
        assertEquals(reader.peek(5), Some("Hello"))

        assertEquals(reader.read(buffer, 0, 5), 5)
        assertEquals(
          buffer.slice(0, 5).toList,
          "Hello".toCharArray().toList
        )

        assertEquals(reader.peek(5), Some(", Wor"))

        assertEquals(reader.read(buffer, 0, 6), 6)
        assertEquals(
          buffer.slice(0, 6).toList,
          ", Worl".toCharArray().toList
        )

        assertEquals(reader.peek(10), Some("d!"))

        assertEquals(reader.read(buffer, 0, 10), 2)
        assertEquals(
          buffer.slice(0, 2).toList,
          "d!".toCharArray().toList
        )

        assertEquals(reader.peek(1), None)
        assertEquals(reader.peek(10), None)
    }

    test("reader read string") {
        val reader = StringReader("Hello, World!")

        assertEquals(reader.readString(5), Some("Hello"))
        assertEquals(reader.readString(3), Some(", W"))
        assertEquals(reader.readString(10), Some("orld!"))
        assertEquals(reader.readString(5), None)
    }
