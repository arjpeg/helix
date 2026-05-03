print "if / else as expressions — the branch value is the result:";

let x = 7;
let label = if x > 0 { "positive" } else if x < 0 { "negative" } else { "zero" };
print label;

print "else if chains:";

fn classify(n) {
    if n < 0      { "negative" }
    else if n == 0 { "zero"     }
    else if n < 10 { "small"    }
    else if n < 100 { "medium"  }
    else           { "large"    }
}

print classify(-3);
print classify(0);
print classify(7);
print classify(42);
print classify(200);

print "if as an expression in a larger computation:";

fn abs(n) { if n < 0 { -n } else { n } }
fn max(a, b) { if a > b { a } else { b } }
fn min(a, b) { if a < b { a } else { b } }

print abs(-9);
print max(4, 11);
print min(4, 11);
