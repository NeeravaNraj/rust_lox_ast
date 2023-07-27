# Lox AST interpreter written in Rust

This is my first project in the Rust programming language.
Thanks to Bob Nystrom for making the lovely book 'Crafting Interpreters'.
There are many quirks in this project and its not super performant either, but this is only for learning purposes.

I have gone ahead and added a lot of extra functionality to the language but not too much either.

Also the syntax itself is a tad bit different compared to the original Lox programming language.
I made it look more like rust.

Now lets go through the syntax of the language!

### To start off lets go with variable binding.
```rust
let a; // uninitialized variable a
let b = 0; // initialized variable b to 0
```


### Comments.
```rust
// Comments can be started with double forward slashes -> '//'

/*
    block comments start with a
    forward slash and star -> '/*'
    and end with star and slash -> */
*/
```

### Mutating variables.
```rust
let a = 0; // initialize variable a to 0
a = 1; // we mutate the value in a to be 1
```
Notice all statements end with a semicolon.

### Arithmetic operators.
```rust
1 + 2; // Add
1 - 2; // Subtract
1 * 2; // Multiply
1 / 2; // Divide
1 % 2; // Modulus
-1; // Negation
// The operaters that are supported are +, -, *, /, %
// I haven't implemented Bitwise operators yet
```

### Compound assignment.
```rust
let a = 0; // initialize variable a to 0
a += 1; // We set a to 'a + 1'
a -= 1; // We set a to 'a - 1'
a *= 1; // We set a to 'a * 1'
a /= 1; // We set a to 'a / 1'
a %= 1; // We set a to 'a % 1'
```
### Logical operators.
```rust
true and false; // And operator
true or false;  // Or operator
!true; // Not operator
1 == 2; // Check equality
1 != 2; // Check inequality
1 > 2; // Greater than operator
1 < 2; // Less than operator
1 >= 2; // Greater equal operator
1 <= 2; // Less equal operator
// All logical operators return booleans
```

### Increment/Decrement operators.
```rust 
let i = 0;
i++; // Postfix increment
i--; // Postfix decrement
++i; // Prefix increment
--i; // Prefix decrement
```

### Ternary operator.
```rust
// The structure for a ternary operator goes as follows
// (condition) ? (return for true) : (return for false)
// For example
1 > 2 ? true : false;
// Here this expression will return false because 1 is not greater than 2
// Values can be of any data type
```


### If Statements
```js
if (2 == 2) {
    // Do stuff
}
```

### If/Else Statements
```js
if (2 == 2) {
    // Do stuff
} else {
    // Do something else
}
```

### If/Elif Statements
```js
if (2 == 2) {
    // Do stuff
} elif {
    //Do something else
}
```

### While Statements
```js
while (i < 10) { // i is some arbitrary value defined by user
    // do stuff
}
```

### For Statements
```js
for (let i = 0 i < 10; i++) { // Usual C style for loops
    // do stuff
}
```

### Break/Continue Statements
```js
for (let i = 0 i < 10; i++) { // Usual C style for loops
    // do stuff
    continue; // Skips current iteration
}

while (i < 10) { // i is some arbitrary value defined by user
    // do stuff
    break; // Exits out of the loop it was encountered in
}
// break/continue can be used in any loop
```

### Functions
```rust
fn greet(name) { // Define a function 'greet' which takes 1 parameter name
    print("Hello", name); // Here 'print' is a native function to the language
    // We call 'print' pass it a string "Hello" and a second parameter name

    // This function will print "Hello {name}" to standard out
}
```

### Calling functions
```rust
fn greet(name) {
    print("Hello", name);
}

// Now to call a function
greet("Rama"); 
/* 
    Type the name followed by the call syntax '()'
    and pass all arguments to the functions in between the parens
*/
```

### Returning from functions
```rust
// Usual function declaration
fn add(a, b) {
    // start a return statement with the return keyword
    return a + b; // this will return the sum of a and b to the caller
}

let a = add(1, 2); // here a will have the value 3

// We can have to return value also as an early return
fn do_nothing() {
    return; // here we immediately return from a function
    // this function doesn't do anything as the name suggests
}
```

### Lambda functions
```rust
let a = lm() {}; // here we declare a lambda function and bind it to a

// Lambda function work similarly to regular functions
let greet = lm(name) {
    print("Hello", name);
};

let sum = lm(a, b) {
    return a + b;
};

// Calling lambda functions
// name of the variable they are bound to followed by the usual call syntax
a(); // does nothing
greet("Jojo"); // prints "Hello Jojo"
let three = sum(1, 2); // returns 3
```

### Classes
```cpp
// To declare classes we start with the class keyword
// followed by an identifier for the class
class Person { // <- class body
    // to declare fields we need to specify if they are public/private or static

    public name; 
    public age;
    // the name and age fields of class 'Person' can be accessed by instances
    private card_number;
    //  the card_number field can only be accessed internally by the class

    // now for the constructor
    init(name, age, card) { 
        // the init method of a class is 
        // always going to be the constructor for the class

        // Now to set fields
        // We will use the 'this' pointer
        this.name = name;
        this.age = age;
        this.card_number = card;
        // we use the dot syntax to get class fields or set them
    }

    // Now for a method
    // declarce a public method named 'print_person'
    public print_person() {
        print(this.name, this.age);
        // we don't want to print the 
        // card_number because thats private information 🤫
    }
}
```

### Class Instantiation
```cpp
class Person {
    public name; 
    public age;
    private card_number;
    init(name, age, card) { 
        this.name = name;
        this.age = age;
        this.card_number = card;
    }

    public print_person() {
        print(this.name, this.age);
    }
}

// To instantiate a class
// we just call it and bind it to a variable
let rama = Person("Rama", 23, 123456);
```

### Accessing class fields/methods
```cpp
class Person {
    public name; 
    public age;
    private card_number;
    init(name, age, card) { 
        this.name = name;
        this.age = age;
        this.card_number = card;
    }

    public print_person() {
        print(this.name, this.age);
    }
}

let akash = Person("Akash", 33, 654321);

// To access the print_person method we defined above
// We use the dot syntax again
akash.print_person();
// We can access the classes fields too!
let name = akash.name;
// but trying to access private fields will result in an error
let card = akash.card_number; // <- error
```

### Static methods/fields
```cpp
// Let's start by declaring a class
class Math {
    // Let's define a static field using the 'static' keyword
    static PI = 3.14;

    // Now let's declare a static method
    static square(n) {
        return n * n;
    }
}
```

### Accessing Static methods/fields
```cpp
class Math {
    static PI = 3.14;

    static square(n) {
        return n * n;
    }
}

let pi = Math.PI; 
// we can directly access the field 'PI' from 'Math' without having to instantiate it

// Same goes for the 'square' method
let sixteen = Math.square(4);
// we can call the square method without instantiating 'Math'
```

## Native functions and methods

- `print(...)` - print whatever is passed to standard out
- `input(str)` - read data from standard in into a string
- `typeof()` - get the typeof any data type
- `clock()` - get current time in unix format

#### Array
- `len()` - returns length of array
- `push(element)` - pushes element that to the end of the array
- `pop()` - pops off the element at the end of the array and returns the value
- `insert(index, value)` - insert a value and index and moves the rest of the elements to the right
- `delete(index)` - deletes the value at index and moves the rest of the elements to the left
- `replace(index, value)` replaces the element at index with value

#### Str
- `len()` - returns length of array
- `slice(start, end)` - extracts a section of a string and returns it as a new string, without modifying the original string
- `split(match)` - splits the string at ever occurence of match and returns an array of the split string
- `replace(match, text)` - matches the first occurence of match in a string and replace it with text and returns the string
- `replacen(match, text, n)` - matches n occurence's of match in a string and replace all of them with text and returns the string
- `toUpper()` - converts string to upper case
- `toLower()` - converts string to lower case
- `trim()` - trims whitespace at beginning and end of a string
- `trim_start()` - trims whitespace at the beginning of a string
- `trim_end()` - trims whitespace at the end of a string

#### Num
- `init(string)` - attempts to parse a string to number, throws error if parsing failed
- **static** `tryParse(string)` - attempts to parse a string to number, instead of throwing an error returns false if attempt failed

#### Data types
- `String`
- `Number`
- `Boolean`
- `none`

#### Data structures 
- `Array`