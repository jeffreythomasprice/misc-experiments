package shared

import (
	"encoding/json"
	"errors"
	"fmt"
)

type ErrorResponse struct {
	Message string `json:"message"`
}

type LoginRequest struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

type LoginResponse struct {
	Message string `json:"message"`
}

type Result[T any] struct {
	Ok  T
	Err ErrorResponse
}

var _ json.Unmarshaler = Result[any]{}

// UnmarshalJSON implements json.Unmarshaler.
func (r Result[T]) UnmarshalJSON(data []byte) error {
	resultErrors := make([]error, 0, 2)
	if err := json.Unmarshal(data, &r.Ok); err != nil {
		resultErrors = append(resultErrors, fmt.Errorf("error trying to unmarshal the ok case: %w", err))
	}
	if err := json.Unmarshal(data, &r.Err); err != nil {
		resultErrors = append(resultErrors, fmt.Errorf("error trying to unmarshal the err case: %w", err))
	}
	return errors.Join(resultErrors...)
}

// TODO unused
/*
UnmarshalledTaggedUnion expects to unmarshal a json object from the given input bytes. It looks for a json key with the name tagName and
matches it against the keys from the given map. Whichever one matches, it unmarshals into that value and returns it. If none match it
returns an error.
*/
func UnmarshalledTaggedUnion(data []byte, tagName string, choices map[string]interface{}) (interface{}, error) {
	keys := make(map[string]json.RawMessage)
	if err := json.Unmarshal(data, &keys); err != nil {
		return nil, err
	}
	tagRaw, ok := keys[tagName]
	if !ok {
		return nil, fmt.Errorf("object missing tag name: %v", tagName)
	}
	var tag string
	if err := json.Unmarshal(tagRaw, &tag); err != nil {
		return nil, fmt.Errorf("not a string value for tag: %v", tagName)
	}
	choice, ok := choices[tag]
	if !ok {
		return nil, fmt.Errorf("not a valid tag: %v", tag)
	}
	if err := json.Unmarshal(data, choice); err != nil {
		return nil, fmt.Errorf("error unmarhsalling input as %v: %w", tag, err)
	}
	return choice, nil
}
