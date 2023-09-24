package dom

import "syscall/js"

type FormData struct {
	js.Value
}

func newFormData(value js.Value) FormData {
	return FormData{value}
}

func (data FormData) Entries() map[string][]js.Value {
	iterator := data.Call("entries")
	results := make(map[string][]js.Value)
	for {
		this := iterator.Call("next")
		if this.Get("done").Truthy() {
			break
		}
		nameValue := this.Get("value")
		name := nameValue.Index(0).String()
		value := nameValue.Index(1)
		results[name] = append(results[name], value)
	}
	return results
}
