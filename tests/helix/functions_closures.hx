fn add(a, b) { a + b }
assert add(2, 3) == 5;
assert add(-1, 1) == 0;

fn factorial(n) {
    if n <= 1 { return 1; };
    n * factorial(n - 1)
}
assert factorial(1) == 1;
assert factorial(5) == 120;

fn make_counter() {
    let count = 0;
    fn() {
        count = count + 1;
        count
    }
}

let c = make_counter();
assert c() == 1;
assert c() == 2;
assert c() == 3;

let a = make_counter();
let b = make_counter();
assert a() == 1;
assert b() == 1;
assert a() == 2;
assert b() == 2;

fn make_adder(n) {
    fn(x) { x + n }
}
let add5 = make_adder(5);
let add10 = make_adder(10);
assert add5(3) == 8;
assert add10(3) == 13;
assert add5(add10(1)) == 16;

fn apply(f, x) { f(x) }
assert apply(make_adder(7), 3) == 10;

fn curry(a) {
    fn(b) { fn(c) { a + b + c } }
}
assert curry(1)(2)(3) == 6;
assert curry(10)(20)(30) == 60;

fn early_return(x) {
    if x > 10 { return 1; };
    if x > 5  { return 2; };
    3
}
assert early_return(20) == 1;
assert early_return(7)  == 2;
assert early_return(3)  == 3;
