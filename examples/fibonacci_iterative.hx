let N = 10;

let previous = 0;
let last = 1;

let i = 0;

while i < N {
    print previous;

    let temp = last;
    last = previous + temp;
    previous = temp;

    i = i + 1;
}

