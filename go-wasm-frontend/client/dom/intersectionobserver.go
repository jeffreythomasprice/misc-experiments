package dom

import "syscall/js"

type IntersectionObserverOptions struct {
	Root       Element
	RootMargin *string
	Threshold  *float64
}

type IntersectionObserver struct {
	value js.Value
}

type IntersectionObserverEntry struct {
	value js.Value
}

func NewIntersectionObserver(callback func(entries []IntersectionObserverEntry, observer IntersectionObserver), options *IntersectionObserverOptions) IntersectionObserver {
	var jsOptions js.Value
	if options != nil {
		jsOptions = js.Global().Get("Object").New()
		if options.Root != nil {
			jsOptions.Set("root", options.Root)
		}
		if options.RootMargin != nil {
			jsOptions.Set("rootMargin", *options.RootMargin)
		}
		if options.Threshold != nil {
			jsOptions.Set("threshold", *options.Threshold)
		}
	}
	return IntersectionObserver{js.Global().Get("IntersectionObserver").New(
		js.FuncOf(func(this js.Value, args []js.Value) any {
			jsEntries := args[0]
			entries := make([]IntersectionObserverEntry, jsEntries.Length())
			for i := 0; i < jsEntries.Length(); i++ {
				entries[i] = IntersectionObserverEntry{jsEntries.Index(i)}
			}
			observer := IntersectionObserver{args[1]}
			callback(entries, observer)
			return nil
		}),
		jsOptions,
	)}
}

func (obs IntersectionObserver) Disconnect() {
	obs.value.Call("disconnect")
}

func (obs IntersectionObserver) Observe(e Element) {
	obs.value.Call("observe", e.jsValue())
}

func (obs IntersectionObserver) Unobserve(e Element) {
	obs.value.Call("unobserve", e.jsValue())
}

func (e IntersectionObserverEntry) Target() Element {
	return newElement(e.value.Get("target"))
}
