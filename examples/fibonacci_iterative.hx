print "fibonacci sequence (iterative):";

let a = 0;
let b = 1;
let i = 0;

while i < 10 {
    print a;
    let next = a + b;
    a = b;
    b = next;
    i = i + 1;
}
