import scala.util.Failure
import scala.util.Try

class StreamsSuite extends munit.FunSuite:
    test("iterable to iterator and back") {
        val stream = IterableStream(List(1, 2, 3, 4, 5))
        val list = stream.iterator.toList
        assertEquals(list, List(1, 2, 3, 4, 5))
    }

    test("callback iterator") {
        var i = 0
        val stream = CallbackStream(() =>
            if i < 5 then
                i += 1
                Some(i)
            else None
        )
        val list = stream.iterator.toList
        assertEquals(list, List(1, 2, 3, 4, 5))
    }

    test("buffered") {
        val stream = BufferedStream(IterableStream(List(1, 2, 3, 4, 5)), 3)
        var saved = stream.position
        assertEquals(stream.peek(), Some(1))
        assertEquals(stream.next(), Some(1))
        assertEquals(stream.peek(), Some(2))
        assertEquals(stream.peek(), Some(2))
        assertEquals(stream.next(), Some(2))
        stream.position = saved
        assertEquals(stream.next(), Some(1))
        assertEquals(stream.next(), Some(2))
        assertEquals(stream.next(), Some(3))
        assertEquals(stream.peek(), Some(4))
        saved = stream.position
        assertEquals(stream.next(), Some(4))
        assertEquals(stream.peek(), Some(5))
        assertEquals(stream.peek(), Some(5))
        assertEquals(stream.next(), Some(5))
        assertEquals(stream.peek(), None)
        assertEquals(stream.next(), None)
        assertEquals(stream.peek(), None)
        stream.position = saved
        assertEquals(stream.peek(), Some(4))
        assertEquals(stream.next(), Some(4))
        assertEquals(stream.peek(), Some(5))
        assertEquals(stream.next(), Some(5))
        assertEquals(stream.peek(), None)
        assertEquals(stream.next(), None)
        assertEquals(stream.peek(), None)
    }

    test("buffered out of range") {
        val stream = BufferedStream(IterableStream(List(1, 2, 3, 4, 5)), 3)
        assertEquals(stream.next(), Some(1))
        val saved = stream.position
        assertEquals(stream.next(), Some(2))
        assertEquals(stream.next(), Some(3))
        assertEquals(stream.next(), Some(4))
        assertEquals(stream.next(), Some(5))
        assertEquals(stream.next(), None)
        assertEquals(
          intercept[IndexOutOfBoundsException] {
              stream.position = saved
          }.getMessage(),
          "position 1 out of bounds, start=3, length=2"
        )
    }
