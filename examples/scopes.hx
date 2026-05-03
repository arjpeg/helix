print "inner blocks can read outer variables:";

let x = 10;
{
    print x;
};

print "let in an inner block shadows without affecting the outer binding:";

{
    let x = 99;
    print x;
};
print x;

print "assignment (without let) reaches through to the outer binding:";

{
    x = 42;
};
print x;

print "shadowing works at every nesting depth independently:";

let depth = 0;
{
    let depth = 1;
    {
        let depth = 2;
        print depth;
    };
    print depth;
};
print depth;

print "functions have their own scope:";

fn make_scope() {
    let local = 100;
    local
}

print make_scope();
