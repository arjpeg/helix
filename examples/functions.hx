fn square(n) { n * n }
fn cube(n)   { n * n * n }
fn abs(n)    { if n < 0 { -n } else { n } }
fn max(a, b) { if a > b { a } else { b } }
fn min(a, b) { if a < b { a } else { b } }

fn clamp(value, lo, hi) {
    max(lo, min(value, hi))
}

fn pow(base, exp) {
    let result = 1;
    let i = 0;
    while i < exp {
        result = result * base;
        i = i + 1;
    }
    result
}

print square(9);
print cube(4);
print abs(-12);
print max(3, 7);
print min(3, 7);
print clamp(15, 0, 10);
print clamp(-5, 0, 10);
print pow(2, 8);
