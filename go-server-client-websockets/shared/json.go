package shared

import (
	"encoding/json"
	"errors"
	"fmt"
	"reflect"
)

type jsonTaggedUnionType struct {
	structType    reflect.Type
	tagField      reflect.StructField
	unionTagValue string
	jsonTagValue  string
}

type JsonTaggedUnion[T any] struct {
	types map[string]jsonTaggedUnionType
}

func NewJsonTaggedUnion[T any](interfaces ...T) (*JsonTaggedUnion[T], error) {
	const UnionTag = "union"

	result := &JsonTaggedUnion[T]{
		types: make(map[string]jsonTaggedUnionType),
	}

	for _, intface := range interfaces {
		t := reflect.TypeOf(intface)
		if t.Kind() == reflect.Pointer {
			t = t.Elem()
		}
		if t.Kind() != reflect.Struct {
			return nil, fmt.Errorf("must be a struct or pointer to struct: %v", t)
		}

		foundIt := false
		for i := 0; i < t.NumField(); i++ {
			field := t.Field(i)
			unionTagValue, ok := field.Tag.Lookup(UnionTag)
			if ok {
				if foundIt {
					return nil, fmt.Errorf("multiple \"%v\" tags on type: %v", UnionTag, t)
				}

				if unionTagValue == "-" {
					unionTagValue = t.Name()
				}

				if _, exists := result.types[unionTagValue]; exists {
					return nil, fmt.Errorf("duplicate union tag: %v", unionTagValue)
				}

				if field.Type.Kind() != reflect.String {
					return nil, fmt.Errorf("field with union tag isn't of type string: %v, %v", t.Name(), field.Name)
				}

				jsonTagValue, ok := field.Tag.Lookup("json")
				if !ok {
					jsonTagValue = field.Name
				}
				result.types[unionTagValue] = jsonTaggedUnionType{
					structType:    t,
					tagField:      field,
					unionTagValue: unionTagValue,
					jsonTagValue:  jsonTagValue,
				}

				foundIt = true
			}
		}
		if !foundIt {
			return nil, fmt.Errorf("type must have a \"%v\" tag: %v", UnionTag, t)
		}
	}

	return result, nil
}

func (instance *JsonTaggedUnion[T]) Marshal(value T) ([]byte, error) {
	impl := func(t jsonTaggedUnionType) ([]byte, error) {
		// get the struct if we have a pointer type
		reflectValue := reflect.ValueOf(value)
		if reflectValue.Kind() == reflect.Pointer {
			reflectValue = reflectValue.Elem()
		}
		// set the field on the struct for the tag to the expected value, if the user forgot to
		reflectValue.FieldByName(t.tagField.Name).SetString(t.unionTagValue)
		// marhsal normally
		return json.Marshal(value)
	}

	// look at all the types
	for _, t := range instance.types {
		// it might be struct, or it might be a pointer to that struct
		valueT := reflect.TypeOf(value)
		if valueT == t.structType {
			return impl(t)
		} else if valueT.Kind() == reflect.Pointer {
			valueT = valueT.Elem()
			if valueT == t.structType {
				return impl(t)
			}
		}
	}
	return nil, fmt.Errorf("no matching type for input: %v", reflect.TypeOf(value))
}

func (instance *JsonTaggedUnion[T]) Unmarshall(b []byte) (T, error) {
	// unmarshal to an object with string keys
	var tmp map[string]json.RawMessage
	if err := json.Unmarshal(b, &tmp); err != nil {
		var emptyReturn T
		return emptyReturn, fmt.Errorf("expected input to be an object with string keys: %v", err)
	}

	// look through all the types
	for _, t := range instance.types {
		// if the json field we're looking for exists on this object
		if rawValueForTag, ok := tmp[t.jsonTagValue]; ok {
			// and it's a string
			var valueForTag string
			if err := json.Unmarshal(rawValueForTag, &valueForTag); err == nil {
				// and it's equal to the expected tag for this type
				if valueForTag == t.unionTagValue {
					// deserialize this one
					result := reflect.New(t.structType).Interface()
					err := json.Unmarshal(b, result)
					if err != nil {
						var emptyReturn T
						return emptyReturn, err
					}
					return result.(T), nil
				}
			}
		}
	}

	var emptyReturn T
	return emptyReturn, errors.New("no tag found on object matching expected types")
}
