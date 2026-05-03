print "boolean operators — and, or, not:";

let a = true;
let b = false;

print a and b;
print a or b;
print !a;
print !(a and b);

print "combining comparisons:";

let x = 15;

print x > 10 and x < 20;
print x < 0 or x > 10;
print !(x == 15);

print "range checks using and:";

fn between(n, lo, hi) {
    n >= lo and n <= hi
}

print between(5, 1, 10);
print between(15, 1, 10);
print between(1, 1, 10);

print "short-circuit behaviour with or chains:";

fn sign(n) {
    if n > 0 { 1 } else { if n < 0 { -1 } else { 0 } }
}

print sign(42);
print sign(-7);
print sign(0);
