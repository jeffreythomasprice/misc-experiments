package shared

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestJsonTaggedUnionMarshalNormalBehavior(t *testing.T) {
	type foo struct {
		Type  string `json:"type" union:"Foo"`
		Value int
	}

	type bar struct {
		Type  string `json:"type" union:"Blah2"`
		Value string
	}

	taggedUnion, err := NewJsonTaggedUnion[interface{}](&foo{}, bar{})
	assert.NoError(t, err)

	jsonBytes, err := taggedUnion.Marshal(&foo{Value: 42})
	assert.NoError(t, err)
	assert.Equal(t, `{"type":"Foo","Value":42}`, string(jsonBytes))

	jsonBytes, err = taggedUnion.Marshal(&bar{Value: "Testing"})
	assert.NoError(t, err)
	assert.Equal(t, `{"type":"Blah2","Value":"Testing"}`, string(jsonBytes))
}

func TestJsonTaggedUnionMarshalDefaultJsonName(t *testing.T) {
	type foo struct {
		Type  string `union:"Foo"`
		Value int
	}

	taggedUnion, err := NewJsonTaggedUnion[interface{}](&foo{})
	assert.NoError(t, err)

	jsonBytes, err := taggedUnion.Marshal(&foo{Value: 42})
	assert.NoError(t, err)
	assert.Equal(t, `{"Type":"Foo","Value":42}`, string(jsonBytes))
}

func TestJsonTaggedUnionMarshalDefaultTagName(t *testing.T) {
	type foo struct {
		Type  string `json:"type" union:"-"`
		Value int
	}

	taggedUnion, err := NewJsonTaggedUnion[interface{}](&foo{})
	assert.NoError(t, err)

	jsonBytes, err := taggedUnion.Marshal(&foo{Value: 42})
	assert.NoError(t, err)
	assert.Equal(t, `{"type":"foo","Value":42}`, string(jsonBytes))
}

func TestJsonTaggedUnionMarshalUnrecognizedType(t *testing.T) {
	type foo struct {
		Type  string `json:"type" union:"Foo"`
		Value int
	}

	type bar struct {
		Type  string `json:"type" union:"Blah2"`
		Value string
	}

	taggedUnion, err := NewJsonTaggedUnion[interface{}](&foo{})
	assert.NoError(t, err)

	_, err = taggedUnion.Marshal(&bar{Value: "testing"})
	assert.ErrorContains(t, err, "no matching type for input: *shared.bar")
}

func TestJsonTaggedUnionUnmarshalNormalBehavior(t *testing.T) {
	type foo struct {
		Type  string `json:"type" union:"Foo"`
		Value int
	}

	type bar struct {
		Type  string `json:"type" union:"Blah2"`
		Value string
	}

	taggedUnion, err := NewJsonTaggedUnion[interface{}](&foo{}, bar{})
	assert.NoError(t, err)

	value, err := taggedUnion.Unmarshall([]byte(`{"type":"Foo","Value":42}`))
	assert.NoError(t, err)
	assert.Equal(t, &foo{Type: "Foo", Value: 42}, value)

	value, err = taggedUnion.Unmarshall([]byte(`{"type":"Blah2","Value":"Testing"}`))
	assert.NoError(t, err)
	assert.Equal(t, &bar{Type: "Blah2", Value: "Testing"}, value)
}

func TestJsonTaggedUnionUnmarshalDefaultJsonName(t *testing.T) {
	type foo struct {
		Type  string `union:"Foo"`
		Value int
	}

	taggedUnion, err := NewJsonTaggedUnion[interface{}](&foo{})
	assert.NoError(t, err)

	value, err := taggedUnion.Unmarshall([]byte(`{"Type":"Foo","Value":42}`))
	assert.NoError(t, err)
	assert.Equal(t, &foo{Type: "Foo", Value: 42}, value)
}

func TestJsonTaggedUnionUnmarshalDefaultTagName(t *testing.T) {
	type foo struct {
		Type  string `json:"type" union:"-"`
		Value int
	}

	taggedUnion, err := NewJsonTaggedUnion[interface{}](&foo{})
	assert.NoError(t, err)

	value, err := taggedUnion.Unmarshall([]byte(`{"type":"foo","Value":42}`))
	assert.NoError(t, err)
	assert.Equal(t, &foo{Type: "foo", Value: 42}, value)
}

func TestJsonTaggedUnionUnmarshalUnrecognizedType(t *testing.T) {
	type foo struct {
		Type  string `json:"type" union:"Foo"`
		Value int
	}

	type bar struct {
		Type  string `json:"type" union:"Blah2"`
		Value string
	}

	taggedUnion, err := NewJsonTaggedUnion[interface{}](&foo{})
	assert.NoError(t, err)

	_, err = taggedUnion.Unmarshall([]byte(`{"type":"Blah2","Value":"Testing"}`))
	assert.ErrorContains(t, err, "no tag found on object matching expected types")
}

func TestJsonTaggedUnionTypeMissingUnionTag(t *testing.T) {
	type foo struct {
		Type  string `json:"type"`
		Value int
	}

	_, err := NewJsonTaggedUnion[interface{}](&foo{})
	assert.ErrorContains(t, err, "type must have a \"union\" tag: shared.foo")
}

func TestJsonTaggedUnionTypeNonStringUnionTag(t *testing.T) {
	type foo struct {
		Type  int `json:"type" union:"Foo"`
		Value int
	}

	_, err := NewJsonTaggedUnion[interface{}](&foo{})
	assert.ErrorContains(t, err, "field with union tag isn't of type string: foo, Type")
}

type commonInterface interface {
	String() string
}

type commonInterfaceImpl1 struct {
	Type  string `json:"type" union:"1"`
	Value int
}

var _ commonInterface = (*commonInterfaceImpl1)(nil)

// String implements commonInterface.
func (x *commonInterfaceImpl1) String() string {
	return fmt.Sprintf("1-%v", x.Value)
}

type commonInterfaceImpl2 struct {
	Type  string `json:"type" union:"2"`
	Value string
}

var _ commonInterface = (*commonInterfaceImpl2)(nil)

// String implements commonInterface.
func (x *commonInterfaceImpl2) String() string {
	return fmt.Sprintf("2-%v", x.Value)
}

func TestJsonTaggedUnionUnmarshalCommonInterface(t *testing.T) {
	taggedUnion, err := NewJsonTaggedUnion[commonInterface](&commonInterfaceImpl1{}, &commonInterfaceImpl2{})
	assert.NoError(t, err)

	value, err := taggedUnion.Unmarshall([]byte(`{"type":"1","Value":42}`))
	assert.NoError(t, err)
	assert.Equal(t, value.String(), "1-42")

	value, err = taggedUnion.Unmarshall([]byte(`{"type":"2","Value":"Testing"}`))
	assert.NoError(t, err)
	assert.Equal(t, value.String(), "2-Testing")
}
