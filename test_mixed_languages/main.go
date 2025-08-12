// Go file
package main

import "fmt"

type Calculator struct {
    value int
}

func NewCalculator(value int) *Calculator {
    return &Calculator{value: value}
}

func (c *Calculator) Add(n int) *Calculator {
    c.value += n
    return c
}

func (c *Calculator) GetValue() int {
    return c.value
}

func fibonacci(n int) int {
    if n <= 1 {
        return n
    }
    return fibonacci(n-1) + fibonacci(n-2)
}

func greet(name string) {
    fmt.Printf("Hello from Go, %s!\n", name)
}

func main() {
    fmt.Println("Hello from Go!")
    calc := NewCalculator(10)
    calc.Add(5)
    fmt.Printf("Result: %d\n", calc.GetValue())
    
    greet("World")
    fmt.Printf("Fibonacci(10): %d\n", fibonacci(10))
}