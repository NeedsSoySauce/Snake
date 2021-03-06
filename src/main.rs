use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode};
use crossterm::ExecutableCommand;
use crossterm::Result;
use crossterm::{cursor, QueueableCommand};

use rand::distributions::{Distribution, Uniform};

use std::env;
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use point::Point;
mod point;

const ROWS: usize = 15;
const COLS: usize = 17;

static mut HORIZONTAL_BORDER: char = '#';
static mut VERTICAL_BORDER: char = '#';
static mut EMPTY_CELL: char = '-';
static mut SNAKE: char = 'S';
static mut FRUIT: char = 'F';
static mut UNKNOWN_CELL_VALUE: char = '?';

const EMPTY_CELL_ID: usize = 0;
const SNAKE_CELL_ID: usize = 1;
const FRUIT_CELL_ID: usize = 2;
const INIT_CELL_ID: usize = 3;

const EXIT_CODE: usize = 0;
const UP_CODE: usize = 1;
const LEFT_CODE: usize = 2;
const DOWN_CODE: usize = 3;
const RIGHT_CODE: usize = 4;

const GAME_SPEED: Duration = Duration::from_millis(100);

type PlayArea = [[usize; ROWS as usize]; COLS as usize];

fn main() {
    println!("{}", env::consts::OS);

    enable_raw_mode().expect("Error enabling raw mode.");

    // Wrap in an ARC so that we can share ownership between the main and second thread
    let input_code = Arc::new(AtomicUsize::new(UP_CODE));
    let exit_flag = Arc::new(AtomicBool::new(false));

    // Create a thread that polls for user input
    let join_handle = thread::spawn({
        let input_code_clone = input_code.clone();
        let exit_flag_clone = exit_flag.clone();

        move || {
            get_input(input_code_clone, exit_flag_clone).expect("Thread error.");
        }
    });

    game_loop(input_code).expect("Error during game loop.");

    exit_flag.store(true, Ordering::Relaxed);
    join_handle.join().expect("Error joining thread.");

    stdout().execute(cursor::MoveDown(1)).unwrap();
    println!("Press any key to exit...\r");
    read().unwrap();

    disable_raw_mode().expect("Error disabling raw mode.");
}

fn get_input(input_code: Arc<AtomicUsize>, exit_flag: Arc<AtomicBool>) -> Result<()> {
    while !exit_flag.load(Ordering::Relaxed) {
        if poll(GAME_SPEED)? {
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
        } else {
            // No input
        }
    }

    Ok(())
}

fn get_random_empty_cell(cells: &mut PlayArea) -> Result<Point> {
    let mut rng = rand::thread_rng();
    let row_die = Uniform::from(0..ROWS);
    let col_die = Uniform::from(0..COLS);

    let mut x: usize;
    let mut y: usize;

    // Keep generating positions until we find an empty cell
    loop {
        x = col_die.sample(&mut rng);
        y = row_die.sample(&mut rng);

        if cells[x][y] == EMPTY_CELL_ID {
            break;
        };
    }

    Ok(Point { x: x, y: y })
}

fn create_fruit(cells: &mut PlayArea) -> Result<Point> {
    let pt = get_random_empty_cell(cells)?;
    cells[pt.x][pt.y] = FRUIT_CELL_ID;
    Ok(pt)
}

fn copy_cells(a: PlayArea, b: &mut PlayArea) {
    for i in 0..ROWS {
        for j in 0..COLS {
            b[j][i] = a[j][i]
        }
    }
}

fn game_loop(input_code: Arc<AtomicUsize>) -> Result<()> {
    let mut cells: PlayArea = [[EMPTY_CELL_ID; ROWS as usize]; COLS as usize];
    let mut prev_cells: PlayArea = [[INIT_CELL_ID; ROWS as usize]; COLS as usize];

    let mut snake: [Point; (ROWS * COLS) as usize] = [Point { x: 0, y: 0 }; (ROWS * COLS) as usize];
    let mut snake_length: usize = 1;

    let mut code = input_code.load(Ordering::Relaxed);

    snake[0].x = COLS / 2;
    snake[0].y = ROWS / 2;

    let mut fruit = create_fruit(&mut cells)?;

    let mut stdout = stdout();

    stdout
        .execute(terminal::Clear(terminal::ClearType::All))?
        .execute(cursor::Hide)?;

    init_screen()?;

    while code != EXIT_CODE {
        let tail = snake[snake_length - 1];

        // The snake's body occupies it's previous position
        // We do need this, but only so we can track the snake's position
        for i in (1..snake_length).rev() {
            snake[i] = snake[i - 1];
        }

        // Move the snake in w/e direction the user last entered
        {
            let mut dest = snake[0];
            let y = dest.y;
            let x = dest.x;

            match code {
                UP_CODE => dest.y = if y == 0 { ROWS - 1 } else { y - 1 },
                LEFT_CODE => dest.x = if x == 0 { COLS - 1 } else { x - 1 },
                DOWN_CODE => dest.y = if y == ROWS - 1 { 0 } else { y + 1 },
                RIGHT_CODE => dest.x = if x == COLS - 1 { 0 } else { x + 1 },
                _ => (),
            }

            // Check if the cell the snake is moving into is occupied by itself
            if cells[dest.x][dest.y] == SNAKE_CELL_ID {
                stdout.execute(cursor::MoveTo(0, (ROWS + 2) as u16))?;
                println!("You came second place!\r");
                break;
            } else {
                let head = &mut snake[0];
                head.x = dest.x;
                head.y = dest.y;
            }
        }

        let head = snake[0];

        // If the snake's head overlaps the current fruit position
        if head.x == fruit.x && head.y == fruit.y {
            snake_length += 1;
            snake[snake_length - 1] = tail;

            if snake_length == (ROWS * COLS) {
                stdout.execute(cursor::MoveTo(0, (ROWS + 2) as u16))?;
                println!("You win!\r");
                break;
            }

            fruit = create_fruit(&mut cells)?;
        } else {
            // Empty the cell at the end of the snake
            cells[tail.x][tail.y] = EMPTY_CELL_ID;
        }

        cells[head.x][head.y] = SNAKE_CELL_ID;

        print_screen(cells, prev_cells)?;
        copy_cells(cells, &mut prev_cells);
        thread::sleep(GAME_SPEED);
        code = input_code.load(Ordering::Relaxed);
    }

    Ok(())
}

fn print_horizontal_border() {
    unsafe {
        for _ in 0..(COLS + 2) {
            print!("{}", HORIZONTAL_BORDER);
        }
    }
    println!("\r");
}

fn init_screen() -> Result<()> {
    let mut stdout = stdout();
    stdout.execute(cursor::MoveTo(0, 0))?;

    print_horizontal_border();
    unsafe {
        for _ in 0..ROWS {
            print!("{}", VERTICAL_BORDER);
            for _ in 0..COLS {
                print!("{}", EMPTY_CELL);
            }
            println!("{}\r", VERTICAL_BORDER);
        }
    }
    print_horizontal_border();
    Ok(())
}

fn print_screen(cells: PlayArea, prev_cells: PlayArea) -> Result<()> {
    let mut stdout = stdout();

    unsafe {
        for i in 0..ROWS {
            for j in 0..COLS {
                let value = cells[j][i];
                let prev_value = prev_cells[j][i];

                if value == prev_value {
                    continue;
                }
                let c: char;

                if value == EMPTY_CELL_ID {
                    c = EMPTY_CELL;
                } else if value == SNAKE_CELL_ID {
                    c = SNAKE;
                } else if value == FRUIT_CELL_ID {
                    c = FRUIT;
                } else {
                    c = UNKNOWN_CELL_VALUE;
                }

                stdout
                    .queue(cursor::MoveTo((j + 1) as u16, (i + 1) as u16))?
                    .queue(Print(c.to_string()))?;
            }
        }
    }

    stdout.flush()?;
    Ok(())
}
