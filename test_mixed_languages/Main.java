// Java file
public class Main {
    public static void main(String[] args) {
        System.out.println("Hello from Java!");
        Calculator calc = new Calculator(10);
        calc.add(5);
    }
}

class Calculator {
    private int value;
    
    public Calculator(int value) {
        this.value = value;
    }
    
    public Calculator add(int n) {
        this.value += n;
        return this;
    }
    
    public int getValue() {
        return this.value;
    }
}