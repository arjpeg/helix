# checks if the given row idx is solved
fn check_row(grid, row) {
    grid[row][0] != "" and grid[row][0] == grid[row][1] and grid[row][1] == grid[row][2]
}

# checks if the given col idx is solved
fn check_col(grid, col) {
    grid[0][col] != "" and grid[0][col] == grid[1][col] and grid[1][col] == grid[2][col]
}

# checks if the left-right diagonal is solved
fn check_lr_diagonal(grid) {
    grid[0][0] != "" and grid[0][0] == grid[1][1] and grid[1][1] == grid[2][2]
}

# checks if the right-left diagonal is solved
fn check_rl_diagonal(grid) {
    grid[0][2] != "" and grid[2][0] == grid[1][1] and grid[1][1] == grid[0][2]
}

# checks if the grid is tied
fn tie(grid) {
    let rows = length(grid);
    let row = 0;

    while row < rows {
        let cols = length(grid[row]);
        let col = 0;

        while col < cols {
            if grid[row][col] == "" {
                return false;
            };

            col = col + 1;
        }

        row = row + 1;
    }

    return true;
}


# prints the grid
fn print_grid(grid) {
    print "Grid: ";

    let rows = length(grid);
    let row = 0;

    while row < rows {
        let cols = length(grid[row]);
        let col = 0;

        let buf = "";

        while col < cols {
            let cell = grid[row][col];

            if cell == "" {
                cell = "_";
            };

            buf = buf + cell + " ";
            col = col + 1;
        }

        print buf;
        row = row + 1;
    }
}

fn winner(grid) {
    let rows = length(grid);
    let cols = length(grid[0]);

    let row = 0;
    while row < rows {
        if check_row(grid, row) {
            return grid[row][0];
        };
        row = row + 1;
    }

    let col = 0;
    while col < cols {
        if check_row(grid, col) {
            return grid[0][col];
        };
        col = col + 1;
    }

    if check_lr_diagonal(grid) {
        return grid[0][0];
    };

    if check_rl_diagonal(grid) {
        return grid[2][0];
    };

    ""
}


print "Welcome to Tic Tic Toe!";

let player_1 = input("Player One, what is your name? ");
let player_2 = input("Player Two, what is your name? ");

let grid = [ ["", "", ""], ["", "", ""], ["", "", ""] ];

let turn = 0;

while winner(grid) == "" and !tie(grid) {
    let symbol = "";

    print_grid(grid);
    print "";

    if turn % 2 == 0 {
        print player_1 + ", it is your turn";
        symbol = "X";
    } else {
        print player_2 + ", it is your turn";
        symbol = "O";
    };

    let row = 0;
    let col = 0;

    while true {
        row = integer(input("Enter a row: "));
        col = integer(input("Enter a column: "));

        if grid[row][col] == "" {
            break;
        } else {
            print "That cell was already taken... Please try again!\n";
        };
    }

    grid[row][col] = symbol;

    print "";

    turn = turn + 1;
}


let player = winner(grid);

if player == "X" {
    print player_1 + " wins!";
} else if player == "O" {
    print player_2 + " wins!";
} else {
    print "Tie!";
};


