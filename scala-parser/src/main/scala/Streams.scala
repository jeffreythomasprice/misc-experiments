import scala.collection.mutable.ArrayDeque
import scala.collection.mutable.ListBuffer
import scala.reflect.ClassTag

trait Stream[T]:
    def next(): Option[T]

trait PeekableStream[T] extends Stream[T]:
    def peek(): Option[T]

class PeekableStreamIterator[T](private val source: PeekableStream[T])
    extends Iterator[T]:
    override def hasNext: Boolean = source.peek().isDefined

    override def next(): T = source.next().get

extension [T](stream: PeekableStream[T])
    def iterator: Iterator[T] = PeekableStreamIterator(stream)

class SingleElementBufferedStream[T](source: Stream[T])
    extends PeekableStream[T]:
    private var _buffer: Option[T] = None

    override def next(): Option[T] =
        val result = buffer()
        _buffer = None
        result

    override def peek(): Option[T] = buffer()

    private def buffer() =
        if _buffer.isEmpty then _buffer = source.next()
        _buffer

extension [T](stream: Stream[T])
    def iterator: Iterator[T] = SingleElementBufferedStream(stream).iterator

class IterableStream[T](source: Iterator[T]) extends Stream[T]:
    def this(source: Iterable[T]) = this(source.iterator)

    override def next(): Option[T] =
        source.nextOption()

class CallbackStream[T](source: () => Option[T]) extends Stream[T]:
    override def next(): Option[T] = source()

class BufferedStream[T: ClassTag](
    source: Stream[T],
    bufferLength: Int = 1024
) extends PeekableStream[T]:

    private var _position: Int = 0
    private val buffer = ArrayDeque[T]()
    private var startOfBuffer = 0

    override def next(): Option[T] =
        peek() match
            case None => None
            case Some(value) =>
                position += 1
                Some(value)

    override def peek(): Option[T] =
        if available == 0 then
            if buffer.length > 0 && buffer.length == bufferLength then
                buffer.removeHead()
                startOfBuffer += 1
            if buffer.length < bufferLength then
                source.next().foreach(buffer.append(_))
        if available > 0 then Some(buffer(position - startOfBuffer))
        else None

    def position: Int = _position

    def position_=(position: Int) =
        if position >= startOfBuffer && position <= endOfBuffer then
            _position = position
        else
            throw IndexOutOfBoundsException(
              s"position $position out of bounds, start=$startOfBuffer, length=${buffer.length}"
            )

    def available: Int =
        if position < endOfBuffer then endOfBuffer - position
        else 0

    private def endOfBuffer: Int =
        startOfBuffer + buffer.length
