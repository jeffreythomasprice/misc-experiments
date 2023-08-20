package main

import (
	"errors"
	"fmt"
	"net/url"
)

type HttpError struct {
	statusCode int
	message    string
}

var _ error = &HttpError{}

func NewHttpError(statusCode int, message string) HttpError {
	return HttpError{
		statusCode,
		message,
	}
}

// Error implements error.
func (e HttpError) Error() string {
	return fmt.Sprintf("HttpError(statusCode=%v, message=%v)", e.statusCode, e.message)
}

func GetStatusCodeForError(err error) int {
	if err == nil {
		return 200
	}
	var httpErr HttpError
	if errors.As(err, &httpErr) {
		return httpErr.statusCode
	} else {
		return 500
	}
}

type FormValidationFunc = func(key string, values []string) error

func MinStringLength(i int) FormValidationFunc {
	return func(key string, values []string) error {
		for _, value := range values {
			if len(value) < i {
				return NewHttpError(400, fmt.Sprintf("all values must be at least %v characters, received \"%v\"", i, value))
			}
		}
		return nil
	}
}

func SingleValue() FormValidationFunc {
	return func(key string, values []string) error {
		if len(values) != 1 {
			return NewHttpError(400, fmt.Sprintf("expected exactly one form value, got %v", len(values)))
		}
		return nil
	}
}

func ValidateFormField(form url.Values, key string, validators ...FormValidationFunc) ([]string, error) {
	value, exists := form[key]
	if !exists {
		return nil, fmt.Errorf("no such key: %v", key)
	}

	for _, validator := range validators {
		if err := validator(key, value); err != nil {
			return nil, fmt.Errorf("validation error for key %v: %v", key, err)
		}
	}

	return value, nil
}

func ValidateSingleFormField(form url.Values, key string, validators ...FormValidationFunc) (string, error) {
	values, err := ValidateFormField(
		form,
		key,
		append([]FormValidationFunc{SingleValue()}, validators...)...,
	)
	if err != nil {
		return "", err
	}
	return values[0], nil
}
