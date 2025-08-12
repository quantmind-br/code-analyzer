// C file
#include <stdio.h>
#include <stdlib.h>

typedef struct {
    int value;
} Calculator;

Calculator* calculator_new(int value) {
    Calculator* calc = malloc(sizeof(Calculator));
    calc->value = value;
    return calc;
}

Calculator* calculator_add(Calculator* calc, int n) {
    calc->value += n;
    return calc;
}

int calculator_get_value(Calculator* calc) {
    return calc->value;
}

int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n-1) + fibonacci(n-2);
}

int main() {
    printf("Hello from C!\n");
    Calculator* calc = calculator_new(10);
    calculator_add(calc, 5);
    printf("Result: %d\n", calculator_get_value(calc));
    free(calc);
    return 0;
}