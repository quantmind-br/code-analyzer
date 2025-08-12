// TypeScript file
interface User {
    name: string;
    age: number;
}

function greetUser(user: User): void {
    console.log(`Hello from TypeScript, ${user.name}!`);
}

class UserManager {
    private users: User[] = [];
    
    addUser(user: User): void {
        this.users.push(user);
    }
    
    getUserCount(): number {
        return this.users.length;
    }
}

export { User, UserManager };