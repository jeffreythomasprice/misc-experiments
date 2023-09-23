package shared

import (
	"encoding/json"
	"fmt"
	"reflect"
)

func UnmarshalTaggedUnionJson(
	tagName string,
	tags map[string]interface{},
	data []byte,
) (interface{}, error) {
	var tagOnly map[string]interface{}
	if err := json.Unmarshal(data, &tagOnly); err != nil {
		return nil, err
	}
	tag, ok := tagOnly[tagName]
	if !ok {
		return nil, fmt.Errorf("json missing tag: %v", tagName)
	}
	tagStr, ok := tag.(string)
	if !ok {
		return nil, fmt.Errorf("json tag wrong type: %v", reflect.TypeOf(tag))
	}
	v, ok := tags[tagStr]
	if !ok {
		return nil, fmt.Errorf("json had tag %v=%v, but no such type specified", tagName, tagStr)
	}
	if err := json.Unmarshal(data, v); err != nil {
		return nil, fmt.Errorf("error unmarshalling tag %v=%v into %v: %w", tagName, tagStr, reflect.TypeOf(v), err)
	}
	return v, nil
}
