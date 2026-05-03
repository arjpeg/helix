print "sum from 1 to 100 (should be 5050):";

let sum = 0;
let i = 1;
while i <= 100 {
    sum = sum + i;
    i = i + 1;
}
print sum;

print "break exits the loop early:";

let n = 0;
while true {
    if n * n > 50 { break; };
    n = n + 1;
}
print n;
print n * n;

print "return exits the enclosing function from inside a loop:";

fn first_above(threshold) {
    let k = 1;
    while true {
        if k * k > threshold { return k; };
        k = k + 1;
    }
    0
}

print first_above(50);
print first_above(200);

print "nested loops — break only exits the inner loop:";

let outer = 1;
while outer <= 3 {
    let inner = 1;
    while inner <= 3 {
        if inner == 3 { break; };
        print outer * 10 + inner;
        inner = inner + 1;
    }
    outer = outer + 1;
}
