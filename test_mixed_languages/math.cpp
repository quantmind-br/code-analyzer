// C++ file  
#include <iostream>
#include <vector>
#include <memory>

class Calculator {
private:
    int value;
    
public:
    Calculator(int v = 0) : value(v) {}
    
    Calculator& add(int n) {
        value += n;
        return *this;
    }
    
    int getValue() const {
        return value;
    }
};

template<typename T>
class Container {
private:
    std::vector<T> items;
    
public:
    void add(const T& item) {
        items.push_back(item);
    }
    
    size_t size() const {
        return items.size();
    }
};

int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n-1) + fibonacci(n-2);
}

int main() {
    std::cout << "Hello from C++!" << std::endl;
    
    Calculator calc(10);
    calc.add(5);
    std::cout << "Result: " << calc.getValue() << std::endl;
    
    Container<int> container;
    container.add(42);
    
    return 0;
}