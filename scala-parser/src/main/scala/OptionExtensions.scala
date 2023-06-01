extension [T](option: Option[T])
    def ifDefined(f: (value: T) => Unit): Option[T] =
        if option.isDefined then f(option.get)
        option

    def ifEmpty(f: () => Unit): Option[T] =
        if option.isEmpty then f()
        option
