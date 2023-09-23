package dom

import "syscall/js"

type SubmitEvent struct {
	*Event
}

func NewSubmitEvent(value js.Value) *SubmitEvent {
	return &SubmitEvent{NewEvent(value)}
}

func (e *SubmitEvent) FormData() *FormData {
	return NewFormData(js.Global().Get("FormData").New(e.Target()))
}
