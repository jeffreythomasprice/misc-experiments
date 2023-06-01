import java.io.BufferedReader
import java.io.Reader

extension (reader: BufferedReader)
    def peek(length: Int): Option[String] =
        reader.mark(length)
        val buffer = new Array[Char](length)
        val result = reader.read(buffer)
        reader.reset()
        if result <= 0 then None
        else Some(new String(buffer, 0, result))

extension (reader: Reader)
    def readString(length: Int): Option[String] =
        val buffer = new Array[Char](length)
        val result = reader.read(buffer)
        if result <= 0 then None
        else Some(new String(buffer, 0, result))
