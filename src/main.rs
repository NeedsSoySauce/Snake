use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::Result;

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const ROWS: i32 = 15;
const COLS: i32 = 17;

const HORIZONTAL_BORDER: char = ' ';
const VERTICAL_BORDER: char = ' ';
const EMPTY_CELL: char = '⬛';
const SNAKE_HEAD: char = '🟩';
const SNAKE_BODY: char = '🟩';
const FRUIT: char = '🟥';
const UNKNOWN_CELL_VALUE: char = '⬜';

const EMPTY_CELL_VALUE: i32 = 0;
const SNAKE_HEAD_VALUE: i32 = 1;
const SNAKE_BODY_VALUE: i32 = 2;
const FRUIT_VALUE: i32 = 3;

const EXIT_CODE: i32 = -1;
const UP_CODE: i32 = 0;
const LEFT_CODE: i32 = 1;
const DOWN_CODE: i32 = 2;
const RIGHT_CODE: i32 = 3;

const GAME_SPEED: Duration = Duration::from_millis(100);

fn main() {
    enable_raw_mode().expect("Error enabling raw mode.");

    //  Wrap in an ARC so that we can share ownership between the main and second thread
    // let input_code = AtomicI32::new(0);
    let input_code = Arc::new(AtomicI32::new(0));

    // Create a thread that polls for user input
    let input_code_clone = input_code.clone();
    let join_handle = thread::spawn(move || {
        poll_input(input_code_clone).expect("Thread error.");
    });

    game_loop(input_code);

    join_handle.join().expect("Error joining thread.");

    disable_raw_mode().expect("Error disabling raw mode.");
}

fn poll_input(input_code: Arc<AtomicI32>) -> Result<()> {
    loop {
        if poll(Duration::from_millis(500))? {
            match read()? {
                Event::Key(event) => {
                    // Exit if CTRL-C is pressed
                    if event.code == KeyCode::Char('c') && event.modifiers == KeyModifiers::CONTROL
                    {
                        input_code.store(EXIT_CODE, Ordering::Relaxed);
                        break;
                    }

                    let mut value = input_code.load(Ordering::Relaxed);

                    match event.code {
                        KeyCode::Char('w') => value = UP_CODE,
                        KeyCode::Char('a') => value = LEFT_CODE,
                        KeyCode::Char('s') => value = DOWN_CODE,
                        KeyCode::Char('d') => value = RIGHT_CODE,
                        _ => (),
                    }

                    input_code.store(value, Ordering::Relaxed);
                }
                _ => (),
            }
        } else {
            // No input
        }
    }

    Ok(())
}

fn game_loop(input_code: Arc<AtomicI32>) {
    let mut cells: [[i32; COLS as usize]; ROWS as usize] = [[0; COLS as usize]; ROWS as usize];

    let mut x: i32 = COLS / 2;
    let mut y: i32 = ROWS / 2;

    let mut code = input_code.load(Ordering::Relaxed);

    while code != EXIT_CODE {
        // Reset the cell we were previously in
        cells[y as usize][x as usize] = 0;

        match code {
            UP_CODE => y -= 1,
            LEFT_CODE => x -= 1,
            DOWN_CODE => y += 1,
            RIGHT_CODE => x += 1,
            _ => (),
        }
        // Wrap x and y around the screen
        if x < 0 {
            x = COLS - 1;
        } else if x > COLS - 1 {
            x = 0;
        }

        if y < 0 {
            y = ROWS - 1;
        } else if y > ROWS - 1 {
            y = 0;
        }

        cells[y as usize][x as usize] = 1;

        // Check if snake head is at the same position as the fruit
        // if so, extend the snake's body and pick a new random fruit location

        print_screen(cells);
        thread::sleep(GAME_SPEED);
        code = input_code.load(Ordering::Relaxed);
    }
}

fn print_horizontal_border() {
    for _ in 0..(COLS + 2) {
        print!("{}", HORIZONTAL_BORDER);
    }
    println!("\r");
}

fn print_screen(cells: [[i32; COLS as usize]; ROWS as usize]) {
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
        println!("{}\r", VERTICAL_BORDER);
    }

    print_horizontal_border();
}
