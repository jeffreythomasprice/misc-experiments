package main

import (
	"errors"
	"fmt"
	"io"
	"strconv"
)

type ASTNode interface {
	Loc() Location
}

type NumberNode struct {
	loc   Location
	value int64
}

var _ ASTNode = (*NumberNode)(nil)

// Loc implements ASTNode.
func (node *NumberNode) Loc() Location {
	return node.loc
}

type AddNode struct {
	left, right ASTNode
}

var _ ASTNode = (*AddNode)(nil)

// Loc implements ASTNode.
func (node *AddNode) Loc() Location {
	return node.left.Loc()
}

type SubtractNode struct {
	left, right ASTNode
}

var _ ASTNode = (*SubtractNode)(nil)

// Loc implements ASTNode.
func (node *SubtractNode) Loc() Location {
	return node.left.Loc()
}

type MultiplyNode struct {
	left, right ASTNode
}

var _ ASTNode = (*MultiplyNode)(nil)

// Loc implements ASTNode.
func (node *MultiplyNode) Loc() Location {
	return node.left.Loc()
}

type DivideNode struct {
	left, right ASTNode
}

var _ ASTNode = (*DivideNode)(nil)

// Loc implements ASTNode.
func (node *DivideNode) Loc() Location {
	return node.left.Loc()
}

type NegateNode struct {
	loc  Location
	node ASTNode
}

var _ ASTNode = (*NegateNode)(nil)

// Loc implements ASTNode.
func (node *NegateNode) Loc() Location {
	return node.loc
}

type ErrUnhandledToken struct {
	T Token
}

var _ error = ErrUnhandledToken{}

// Error implements error.
func (err ErrUnhandledToken) Error() string {
	return fmt.Sprintf("unhandled token %v", err.T)
}

type ErrExpectedToken struct {
	T TokenType
}

var _ error = ErrExpectedToken{}

// Error implements error.
func (err ErrExpectedToken) Error() string {
	return fmt.Sprintf("expected token %v", err.T)
}

func Parse(input []Token) (ASTNode, error) {
	result, remainder, err := parseExpression(input)
	if err != nil {
		return nil, err
	}
	if len(remainder) > 0 {
		return nil, ErrUnhandledToken{remainder[0]}
	}
	return result, nil
}

func parseExpression(input []Token) (ASTNode, []Token, error) {
	return parseAddAndSubtract(input)
}

func parseAddAndSubtract(input []Token) (ASTNode, []Token, error) {
	first, remainder, err := parseMultiplyAndDivide(input)
	if err != nil {
		return nil, input, err
	}
	result := first
	for len(remainder) > 0 {
		op := remainder[0].Type
		if op != TokPlus && op != TokMinus {
			break
		}
		remainder = remainder[1:]
		var next ASTNode
		next, remainder, err = parseMultiplyAndDivide(remainder)
		if err != nil {
			return nil, input, err
		}
		if op == TokPlus {
			result = &AddNode{
				left:  result,
				right: next,
			}
		} else {
			result = &SubtractNode{
				left:  result,
				right: next,
			}
		}
	}
	return result, remainder, nil
}

func parseMultiplyAndDivide(input []Token) (ASTNode, []Token, error) {
	first, remainder, err := parseParenthesisOrNegateOrJustNumber(input)
	if err != nil {
		return nil, input, err
	}
	result := first
	for len(remainder) > 0 {
		op := remainder[0].Type
		if op != TokAsterisk && op != TokSlash {
			break
		}
		remainder = remainder[1:]
		var next ASTNode
		next, remainder, err = parseParenthesisOrNegateOrJustNumber(remainder)
		if err != nil {
			return nil, input, err
		}
		if op == TokAsterisk {
			result = &MultiplyNode{
				left:  result,
				right: next,
			}
		} else {
			result = &DivideNode{
				left:  result,
				right: next,
			}
		}
	}
	return result, remainder, nil
}

func parseParenthesisOrNegateOrJustNumber(input []Token) (ASTNode, []Token, error) {
	return parseOneOf(
		input,
		parseParenthesis,
		parseNegate,
		parseNumber,
	)
}

func parseParenthesis(input []Token) (ASTNode, []Token, error) {
	remainder := input
	var err error
	_, remainder, err = expectToken(remainder, TokLeftParen)
	if err != nil {
		return nil, input, err
	}
	var result ASTNode
	result, remainder, err = parseExpression(remainder)
	if err != nil {
		return nil, input, err
	}
	_, remainder, err = expectToken(remainder, TokRightParen)
	if err != nil {
		return nil, input, err
	}
	return result, remainder, nil
}

func parseNegate(input []Token) (ASTNode, []Token, error) {
	remainder := input
	var negateToken Token
	var err error
	negateToken, remainder, err = expectToken(remainder, TokMinus)
	if err != nil {
		return nil, input, err
	}
	var result ASTNode
	result, remainder, err = parseExpression(remainder)
	if err != nil {
		return nil, input, err
	}
	return &NegateNode{
		loc:  negateToken.Loc,
		node: result,
	}, remainder, nil
}

func parseNumber(input []Token) (ASTNode, []Token, error) {
	tok, remainder, err := expectToken(input, TokNumber)
	if err != nil {
		return nil, input, err
	}
	value, err := strconv.ParseInt(tok.Value, 10, 64)
	if err != nil {
		return nil, input, fmt.Errorf("failed to parse number at %v: %w", tok, err)
	}
	return &NumberNode{
		loc:   tok.Loc,
		value: value,
	}, remainder, nil
}

func parseOneOf(input []Token, parsers ...func(input []Token) (ASTNode, []Token, error)) (ASTNode, []Token, error) {
	errs := make([]error, 0, len(parsers))
	for _, p := range parsers {
		result, remainder, err := p(input)
		if err == nil {
			return result, remainder, nil
		}
		errs = append(errs, err)
	}
	if len(input) == 0 {
		errs = append(errs, io.EOF)
	} else {
		errs = append(errs, ErrUnhandledToken{input[0]})
	}
	return nil, input, errors.Join(errs...)
}

func expectToken(input []Token, t TokenType) (Token, []Token, error) {
	if len(input) == 0 {
		return Token{}, input, ErrExpectedToken{T: t}
	}
	if input[0].Type != t {
		return Token{}, input, ErrExpectedToken{T: t}
	}
	return input[0], input[1:], nil
}
