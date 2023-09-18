package dom

import "syscall/js"

type EventTarget interface {
	AddEventListener(typ string, listener func(args []js.Value))
}
