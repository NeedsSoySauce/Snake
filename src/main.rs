const ROWS: usize = 15;
const COLS: usize = 17;

const HORIZONTAL_BORDER: char = '#';
const VERTICAL_BORDER: char = '#';
const EMPTY_CELL: char = ' ';

fn main() {
    println!("Hello, world!");

    let mut cells = [[0; COLS]; ROWS];

    cells[0][0] = 5;
    print_screen(cells);
}

fn print_horizontal_border() {
    for _ in 0..(COLS + 2) {
        print!("{}", HORIZONTAL_BORDER);
    }
    println!("");
}

fn print_screen(cells: [[i32; COLS]; ROWS]) {
    print_horizontal_border();

    for row in cells.iter() {
        print!("{}", VERTICAL_BORDER);
        for col in row.iter() {
            let value = *col;
            if value == 0 {
                print!("{}", EMPTY_CELL);
            } else {
                print!("{}", value);
            }
        }
        println!("{}", VERTICAL_BORDER);
    }

    print_horizontal_border();
}
