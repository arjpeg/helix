print "make_adder returns a closure that adds n to its argument:";

fn make_adder(n) {
    fn(x) { x + n }
}

let add5  = make_adder(5);
let add10 = make_adder(10);

print add5(3);
print add10(3);
print add5(add10(1));

print "closures can be passed to and returned from functions:";

fn apply(f, x) { f(x) }
fn compose(f, g) { fn(x) { f(g(x)) } }

let add15 = compose(add5, add10);
print apply(add15, 0);
print apply(add15, 7);
