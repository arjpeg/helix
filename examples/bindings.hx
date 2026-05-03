print "let declares a new binding:";

let x = 10;
let greeting = "hello";
let flag = true;

print x;
print greeting;
print flag;

print "assignment updates an existing binding:";

x = 99;
print x;

print "blocks are expressions — their tail value is what they evaluate to:";

let distance = {
    let dx = 3;
    let dy = 4;
    dx * dx + dy * dy
};
print distance;

print "lambdas are values and can be stored in bindings:";

let double = fn(n) { n * 2 };
let square = fn(n) { n * n };

print double(7);
print square(7);
print double(square(3));
