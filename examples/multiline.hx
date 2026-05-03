print "blocks evaluate to their tail — the last expression without a semicolon:";

let a = {
    let x = 3;
    let y = 4;
    x * x + y * y
};
print a;

print "intermediate expressions with semicolons are evaluated and discarded:";

let b = {
    10 * 10;
    20 * 20;
    30 * 30
};
print b;

print "blocks compose with the rest of an expression:";

print { 1 + 6 } / 5;
print { 10 } + { 20 };

print "nested blocks:";

let result = {
    let outer = {
        let inner = 5;
        inner * 2
    };
    outer + 1
};
print result;
