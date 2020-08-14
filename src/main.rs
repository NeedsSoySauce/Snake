use std::thread;
use std::time::Duration;

const ROWS: usize = 15;
const COLS: usize = 17;

const HORIZONTAL_BORDER: char = ' ';
const VERTICAL_BORDER: char = ' ';
const EMPTY_CELL: char = '⬛';
const SNAKE_HEAD: char = '🟩';
const SNAKE_BODY: char = '🟩';
const FRUIT: char = '🟥';
const UNKNOWN_CELL_VALUE: char = '⬜';

const EMPTY_CELL_VALUE: u32 = 0;
const SNAKE_HEAD_VALUE: u32 = 1;
const SNAKE_BODY_VALUE: u32 = 2;
const FRUIT_VALUE: u32 = 3;

const GAME_SPEED: Duration = Duration::from_millis(333);

fn main() {
    game_loop();
}

fn game_loop() {
    let mut cells: [[u32; COLS]; ROWS] = [[0; COLS]; ROWS];
    let mut i = 0;
    loop {
        cells[0][i] = 1;
        i += 1;

        // Check if snake head is at the same position as the fruit
        // if so, generate a new fruit and extend the snake's body

        if i > 10 {
            break;
        }

        print_screen(cells);
        thread::sleep(GAME_SPEED);
    }
}

fn print_horizontal_border() {
    for _ in 0..(COLS + 2) {
        print!("{}", HORIZONTAL_BORDER);
    }
    println!("");
}

fn print_screen(cells: [[u32; COLS]; ROWS]) {
    // Clear terminal
    // See: http://rosettacode.org/wiki/Terminal_control/Clear_the_screen#Rust
    print!("{}[2J", 27 as char);

    print_horizontal_border();

    for row in cells.iter() {
        print!("{}", VERTICAL_BORDER);
        for col in row.iter() {
            let value = *col;

            if value == EMPTY_CELL_VALUE {
                print!("{}", EMPTY_CELL);
            } else if value == SNAKE_HEAD_VALUE {
                print!("{}", SNAKE_HEAD);
            } else if value == SNAKE_BODY_VALUE {
                print!("{}", SNAKE_BODY);
            } else if value == FRUIT_VALUE {
                print!("{}", FRUIT);
            } else {
                print!("{}", UNKNOWN_CELL_VALUE);
            }
        }
        println!("{}", VERTICAL_BORDER);
    }

    print_horizontal_border();
}
