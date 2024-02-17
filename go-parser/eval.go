package main

import "fmt"

func Eval(node ASTNode) (int64, error) {
	switch n := node.(type) {
	case *NumberNode:
		return n.value, nil

	case *AddNode:
		var left, right int64
		var err error
		left, err = Eval(n.left)
		if err != nil {
			return 0, err
		}
		right, err = Eval(n.right)
		if err != nil {
			return 0, err
		}
		return left + right, nil

	case *SubtractNode:
		var left, right int64
		var err error
		left, err = Eval(n.left)
		if err != nil {
			return 0, err
		}
		right, err = Eval(n.right)
		if err != nil {
			return 0, err
		}
		return left - right, nil

	case *MultiplyNode:
		var left, right int64
		var err error
		left, err = Eval(n.left)
		if err != nil {
			return 0, err
		}
		right, err = Eval(n.right)
		if err != nil {
			return 0, err
		}
		return left * right, nil

	case *DivideNode:
		var left, right int64
		var err error
		left, err = Eval(n.left)
		if err != nil {
			return 0, err
		}
		right, err = Eval(n.right)
		if err != nil {
			return 0, err
		}
		return left / right, nil

	case *NegateNode:
		result, err := Eval(n.node)
		if err != nil {
			return 0, err
		}
		return -result, nil

	default:
		return 0, fmt.Errorf("unhandled expression at %v: %v", node.Loc(), node)
	}
}
