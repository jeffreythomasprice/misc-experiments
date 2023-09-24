package dom

import "syscall/js"

type SubmitEvent interface {
	Event
	FormData() FormData
}

type submitEventImpl struct {
	eventImpl
}

var _ SubmitEvent = submitEventImpl{}

func newSubmitEvent(value js.Value) submitEventImpl {
	return submitEventImpl{newEvent(value)}
}

func (e submitEventImpl) FormData() FormData {
	return newFormData(js.Global().Get("FormData").New(e.Target()))
}
