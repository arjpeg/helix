if true { assert true; };
if false { assert false; };

if false {
    assert false;
} else {
    assert true;
};

let x = 5;
if x > 3 {
    assert true;
} else {
    assert false;
};

let grade = 85;
let label = if grade >= 90 {
    "A"
} else if grade >= 80 {
    "B"
} else if grade >= 70 {
    "C"
} else {
    "F"
};
assert label == "B";

assert if true { 1 } else { 2 } == 1;
assert if false { 1 } else { 2 } == 2;

let a = 3;
let b = 7;
let max = if a > b { a } else { b };
assert max == 7;

let n = 4;
let kind = if n == 0 {
    "zero"
} else if n * n == n {
    "one"
} else {
    "other"
};
assert kind == "other";

fn abs(v) {
    if v < 0 { -v } else { v }
}
assert abs(-5) == 5;
assert abs(5) == 5;
assert abs(0) == 0;
