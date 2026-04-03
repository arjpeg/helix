let age = 18;

if age > 18 {
    assert false;
} else if age == 18 {
    print age;
} else {
    assert false;
};

let b = if age == 10 { 5 } else if age == 21 { 4 } else { 3 };

print b;

