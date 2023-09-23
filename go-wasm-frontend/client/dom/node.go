package dom

import "syscall/js"

type Node struct {
	js.Value
}

func NewNode(value js.Value) *Node {
	return &Node{value}
}

func (n *Node) AppendChild(other *Node) {
	n.Call("appendChild", other.Value)
}

func (n *Node) RemoveChild(other *Node) {
	n.Call("removeChild", other.Value)
}
