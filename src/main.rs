use crossterm::event::{read, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::Result;

use rand::distributions::{Distribution, Uniform};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use point::Point;
mod point;

const ROWS: usize = 15;
const COLS: usize = 17;

const HORIZONTAL_BORDER: char = ' ';
const VERTICAL_BORDER: char = ' ';
const EMPTY_CELL: char = '⬛';
const SNAKE: char = '🟩';
const FRUIT: char = '🟥';
const UNKNOWN_CELL_VALUE: char = '⬜';

const EMPTY_CELL_ID: usize = 0;
const SNAKE_CELL_ID: usize = 1;
const FRUIT_CELL_ID: usize = 2;

const EXIT_CODE: usize = 0;
const UP_CODE: usize = 1;
const LEFT_CODE: usize = 2;
const DOWN_CODE: usize = 3;
const RIGHT_CODE: usize = 4;

const GAME_SPEED: Duration = Duration::from_millis(100);

fn main() {
    enable_raw_mode().expect("Error enabling raw mode.");

    // Wrap in an ARC so that we can share ownership between the main and second thread
    let input_code = Arc::new(AtomicUsize::new(UP_CODE));

    // Create a thread that polls for user input
    let input_code_clone = input_code.clone();
    let join_handle = thread::spawn(move || {
        get_input(input_code_clone).expect("Thread error.");
    });

    game_loop(input_code);

    join_handle.join().expect("Error joining thread.");

    disable_raw_mode().expect("Error disabling raw mode.");
}

fn get_input(input_code: Arc<AtomicUsize>) -> Result<()> {
    loop {
        match read()? {
            Event::Key(event) => {
                // Exit if CTRL-C is pressed
                if event.code == KeyCode::Char('c') && event.modifiers == KeyModifiers::CONTROL {
                    input_code.store(EXIT_CODE, Ordering::Relaxed);
                    break;
                }

                let mut value = input_code.load(Ordering::Relaxed);

                match event.code {
                    KeyCode::Char('w') => value = UP_CODE,
                    KeyCode::Char('W') => value = UP_CODE,
                    KeyCode::Up => value = UP_CODE,
                    KeyCode::Char('a') => value = LEFT_CODE,
                    KeyCode::Char('A') => value = LEFT_CODE,
                    KeyCode::Left => value = LEFT_CODE,
                    KeyCode::Char('s') => value = DOWN_CODE,
                    KeyCode::Char('S') => value = DOWN_CODE,
                    KeyCode::Down => value = DOWN_CODE,
                    KeyCode::Char('d') => value = RIGHT_CODE,
                    KeyCode::Char('D') => value = RIGHT_CODE,
                    KeyCode::Right => value = RIGHT_CODE,
                    _ => (),
                }

                input_code.store(value, Ordering::Relaxed);
            }
            _ => (),
        }
    }

    Ok(())
}

fn get_random_empty_cell(cells: &mut [[usize; ROWS as usize]; COLS as usize]) -> Point {
    let mut rng = rand::thread_rng();
    let row_die = Uniform::from(0..ROWS);
    let col_die = Uniform::from(0..COLS);

    let mut x: usize;
    let mut y: usize;

    // Keep generating positions until we find an empty cell
    let mut count = 0;
    loop {
        x = col_die.sample(&mut rng);
        y = row_die.sample(&mut rng);
        count += 1;

        if cells[x][y] == EMPTY_CELL_ID {
            break;
        };

        if count > ROWS * COLS {
            panic!("No empty cells available!");
        }
    }

    return Point { x: x, y: y };
}

fn create_fruit(cells: &mut [[usize; ROWS as usize]; COLS as usize]) -> Point {
    let pt = get_random_empty_cell(cells);
    cells[pt.x][pt.y] = FRUIT_CELL_ID;
    return pt;
}

fn game_loop(input_code: Arc<AtomicUsize>) {
    let mut cells: [[usize; ROWS as usize]; COLS as usize] = [[0; ROWS as usize]; COLS as usize];

    let mut snake: [Point; (ROWS * COLS) as usize] = [Point { x: 0, y: 0 }; (ROWS * COLS) as usize];
    let mut snake_length: usize = 1;

    let mut code = input_code.load(Ordering::Relaxed);

    snake[0].x = COLS / 2;
    snake[0].y = ROWS / 2;

    let mut fruit = create_fruit(&mut cells);

    while code != EXIT_CODE {
        let tail = snake[snake_length - 1];

        // The snake's body occupies it's previous position
        // We do need this, but only so we can track the snake's position
        for i in (1..snake_length).rev() {
            snake[i] = snake[i - 1];
        }

        // Move the snake in w/e direction the user last entered
        {
            let head = &mut snake[0];
            let y = head.y;
            let x = head.x;

            match code {
                UP_CODE => head.y = if y == 0 { ROWS - 1 } else { y - 1 },
                LEFT_CODE => head.x = if x == 0 { COLS - 1 } else { x - 1 },
                DOWN_CODE => head.y = if y == ROWS - 1 { 0 } else { y + 1 },
                RIGHT_CODE => head.x = if x == COLS - 1 { 0 } else { x + 1 },
                _ => (),
            }
        }

        let head = snake[0];

        // If the snake's head overlaps the current fruit position
        if head.x == fruit.x && head.y == fruit.y {
            fruit = create_fruit(&mut cells);
            snake_length += 1;
            snake[snake_length - 1] = tail;

            if snake_length > (ROWS * COLS) {
                println!("You win!\r");
                return;
            }
        } else {
            // Empty the cell at the end of the snake
            cells[tail.x][tail.y] = EMPTY_CELL_ID;
        }

        cells[head.x][head.y] = SNAKE_CELL_ID;

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

fn print_screen(cells: [[usize; ROWS as usize]; COLS as usize]) {
    // Clear terminal
    // See: http://rosettacode.org/wiki/Terminal_control/Clear_the_screen#Rust
    print!("{}[2J", 27 as char);

    print_horizontal_border();

    for i in 0..ROWS {
        print!("{}", VERTICAL_BORDER);
        for j in 0..COLS {
            let value = cells[j][i];

            if value == EMPTY_CELL_ID {
                print!("{}", EMPTY_CELL);
            } else if value == SNAKE_CELL_ID {
                print!("{}", SNAKE);
            } else if value == FRUIT_CELL_ID {
                print!("{}", FRUIT);
            } else {
                print!("{}", UNKNOWN_CELL_VALUE);
            }
        }
        println!("{}\r", VERTICAL_BORDER);
    }

    print_horizontal_border();
}
