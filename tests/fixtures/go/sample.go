package demo

import (
	"fmt"
	ioalias "io"
)

type Base struct{}

type Person struct {
	Name string `json:"name"`
	Age  int
	Base
}

type Greeter interface {
	Greet() string
	Read(p []byte) (n int, err error)
}

const Version = "1.0.0"

var internalCounter = 1

func Add(a int, b int) int {
	_ = ioalias.Reader(nil)
	return a + b
}

func (p *Person) Greet() string {
	return fmt.Sprintf("hello %s", p.Name)
}
