# Python file
def greet(name):
    print(f"Hello from Python, {name}!")

class Calculator:
    def __init__(self, value=0):
        self.value = value
    
    def add(self, n):
        self.value += n
        return self
    
    def get_value(self):
        return self.value

def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)