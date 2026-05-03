let i = 0;
let sum = 0;
while 5 > i {
    sum = sum + i;
    i = i + 1;
}
assert sum == 10;

let j = 0;
while 100 > j {
    if j == 7 { break; };
    j = j + 1;
}
assert j == 7;

let total = 0;
let outer = 0;
while 3 > outer {
    let inner = 0;
    while 10 > inner {
        if inner == 2 { break; };
        total = total + 1;
        inner = inner + 1;
    }
    outer = outer + 1;
}
assert total == 6;

fn find(target) {
    let k = 0;
    while 100 > k {
        if k == target { return k; };
        k = k + 1;
    }
    9999
}
assert find(42) == 42;

fn returns_from_loop() {
    let n = 0;
    while 100 > n {
        if n == 3 { return 777; };
        n = n + 1;
    }
    0
}
assert returns_from_loop() == 777;

fn inner_breaks() {
    let x = 0;
    while 5 > x {
        if x == 2 { break; };
        x = x + 1;
    }
    x
}
let calls = 0;
let o = 0;
while 3 > o {
    assert inner_breaks() == 2;
    calls = calls + 1;
    o = o + 1;
}
assert calls == 3;

fn early(p) {
    if p { return 1; };
    2
}
assert early(true) == 1;
assert early(false) == 2;

print "ok";
