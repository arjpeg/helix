let i = 0;
let sum = 0;
while i < 5 {
    sum = sum + i;
    i = i + 1;
}
assert i == 5;
assert sum == 10;

let n = 0;
while n < 10 {
    if n == 4 { break; };
    n = n + 1;
}
assert n == 4;

let outer = 0;
let total = 0;
while outer < 3 {
    let inner = 0;
    while inner < 3 {
        if inner == 2 { break; };
        total = total + 1;
        inner = inner + 1;
    }
    outer = outer + 1;
}
assert total == 6;

fn sum_to(limit) {
    let acc = 0;
    let k = 0;
    while k < limit {
        acc = acc + k;
        k = k + 1;
    }
    acc
}
assert sum_to(5) == 10;
assert sum_to(0) == 0;
assert sum_to(1) == 0;

fn first_multiple(factor) {
    let k = 1;
    while true {
        if k * factor > 20 { return k; };
        k = k + 1;
    }
    0
}
assert first_multiple(7) == 3;
assert first_multiple(11) == 2;
