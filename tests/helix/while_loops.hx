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

# continue skips the rest of the loop body and jumps to the next iteration
let i = 0;
let count = 0;
while i < 5 {
    i = i + 1;
    if i == 3 { continue; };
    count = count + 1;
}
assert count == 4;

# use integer division trick to detect even: n/2*2 == n
let i = 0;
let sum = 0;
while i < 10 {
    i = i + 1;
    if i / 2 * 2 != i { continue; };
    sum = sum + i;
}
assert sum == 30;

# continue only exits the innermost loop
let outer = 0;
let count = 0;
while outer < 3 {
    outer = outer + 1;
    let inner = 0;
    while inner < 4 {
        inner = inner + 1;
        if inner == 2 { continue; };
        count = count + 1;
    }
}
assert count == 9;

# continue and break can coexist in the same loop
let i = 0;
let sum = 0;
while true {
    i = i + 1;
    if i > 10 { break; };
    if i / 2 * 2 != i { continue; };
    sum = sum + i;
}
assert sum == 30;

# continue inside a function
fn sum_evens(limit) {
    let acc = 0;
    let k = 0;
    while k < limit {
        k = k + 1;
        if k / 2 * 2 != k { continue; };
        acc = acc + k;
    }
    acc
}
assert sum_evens(6) == 12;
assert sum_evens(10) == 30;
