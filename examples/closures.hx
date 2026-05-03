fn make_counter() {
    let count = 0;
    fn() {
        count = count + 1;
        count
    }
}

let a = make_counter();
let b = make_counter();
assert a() == 1;
assert a() == 2;
assert b() == 1;
assert a() == 3;
assert b() == 2;

fn curry_add(a) {
    fn(b) {
        fn(c) { a + b + c }
    }
}
assert curry_add(1)(2)(3) == 6;
assert curry_add(10)(20)(30) == 60;

fn make_toggler() {
    let on = false;
    fn() {
        on = !on;
        on
    }
}

let t = make_toggler();
assert t() == true;
assert t() == false;
assert t() == true;
