fn make_adder(n) {
    fn(x) { x + n }
}

let add5 = make_adder(5);
let add10 = make_adder(10);

assert add5(3) == 8;
assert add10(3) == 13;
assert add5(add10(1)) == 16;
