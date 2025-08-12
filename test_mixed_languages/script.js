// JavaScript file
function greet(name) {
    console.log(`Hello from JavaScript, ${name}!`);
}

class Calculator {
    constructor(value = 0) {
        this.value = value;
    }
    
    add(n) {
        this.value += n;
        return this;
    }
    
    getValue() {
        return this.value;
    }
}

export { Calculator };