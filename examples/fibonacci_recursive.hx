fn fibonacci(n) {
    if n <= 1 {
        n
    } else {
        fibonacci(n-1) + fibonacci(n-2)
    }
}

let i = 0;

while i < 100 {
    print fibonacci(i);
    i = i + 1;
}

