package dom

import (
	"errors"
	"fmt"
	"strings"
	"syscall/js"
)

type JsException struct {
	js.Value
	Message string
	Stack   string
}

var _ error = JsException{}

// Error implements error.
func (err JsException) Error() string {
	// make sure lines of stack trace are slightly indented
	lines := strings.Split(err.Stack, "\n")
	for i, line := range lines {
		if !strings.HasPrefix(line, "\t") {
			lines[i] = fmt.Sprintf("\t%s", line)
		}
	}

	return fmt.Sprintf("%s\n%s", err.Message, strings.Join(lines, "\n"))
}

func commonErrorHandling(err *error) {
	if r := recover(); r != nil {
		// see if we can get an actual javascript exception with a stack trace out of it
		if rAsError, ok := r.(error); ok {
			var jsErr js.Error
			if errors.As(rAsError, &jsErr) {
				*err = JsException{
					Value:   jsErr.Value,
					Message: jsErr.Get("message").String(),
					Stack:   jsErr.Get("stack").String(),
				}
				return
			}
		}

		// any other kind of error just do some kind of string representation
		*err = errors.New(fmt.Sprintf("%v", r))
	}
}
