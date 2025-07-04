package main

import (
	"fmt"
	"unsafe"

	"github.com/cogentcore/webgpu/wgpu"
)

type Buffer[T any] struct {
	Buffer        *wgpu.Buffer
	StrideInBytes uintptr
	Length        int
}

func NewBufferInit[T any](device *wgpu.Device, data []T, usage wgpu.BufferUsage) (*Buffer[T], error) {
	result, err := device.CreateBufferInit(&wgpu.BufferInitDescriptor{
		Contents: toByteSlice(data),
		Usage:    usage | wgpu.BufferUsageCopyDst,
	})
	if err != nil {
		return nil, err
	}
	return &Buffer[T]{
		Buffer:        result,
		StrideInBytes: unsafe.Sizeof(data[0]),
		Length:        len(data),
	}, nil
}

func (b *Buffer[T]) Destroy() {
	b.Buffer.Release()
	b.Buffer = nil
}

func toByteSlice[T any](in []T) []byte {
	fmt.Printf("TODO toByteSlice: %v, len=%v, sizeof[0]=%v", in, len(in), unsafe.Sizeof(in[0]))
	return unsafe.Slice((*byte)(unsafe.Pointer(&in[0])), uintptr(len(in))*unsafe.Sizeof(in[0]))
}
