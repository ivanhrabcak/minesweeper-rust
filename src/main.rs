use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use terminal_size::{Height, Width, terminal_size};
use std::{io::{Write, stdin, stdout}, mem};
use console::Term;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub size_x: i32,
    pub size_y: i32
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Square {
    Mine,
    Num(i16),
    Nothing,
    Revealed
}

#[derive(Debug, Clone)]
struct Field {
    pub seed: u64,
    squares: Vec<Square>,
    shown_squares: Vec<Square>,
    size: Size
}

fn replace<T>(vec: &mut Vec<T>, i: usize, e: T) -> T {
    mem::replace(&mut vec[i], e)
}

fn string_repeat(s: &str, times: i32) -> String {
    let mut output = String::new();
    for _ in 0..times {
        output += s;
    }

    output
}  

impl Field {
    pub fn new(size: Size, mines: i32, seed: u64) -> Self { // supply 0 for a random seed
        let mut field = Field { squares: Vec::new(), shown_squares: Vec::new(), size, seed };
        field.init(mines);
        field
    }
    
    pub fn pos_to_index(&self, pos: Position) -> usize {
        (pos.x * self.size.size_x + pos.y) as usize 
    }

    fn init(&mut self, mines: i32) {
        let field_size = self.size.size_x * self.size.size_y;
    
        for _ in 0..field_size {
            self.squares.push(Square::Nothing);
        }

        self.shown_squares = self.squares.clone();
        
        let mut rng: Pcg64 = match self.seed {
            0 => Pcg64::from_entropy(),
            i => Pcg64::seed_from_u64(i)
        };
        
        for _ in 0..mines {
            let mut new_mine_pos: Position = Position { x: rng.gen_range(0..self.size.size_x), y: rng.gen_range(0..self.size.size_y) };
            
            let mut new_mine_index = self.pos_to_index(new_mine_pos); 
            while self.squares.get(new_mine_index).unwrap() != &Square::Nothing {
                new_mine_pos = Position { x: rng.gen_range(0..self.size.size_x), y: rng.gen_range(0..self.size.size_y) };
                new_mine_index = self.pos_to_index(new_mine_pos); 
            }
            
            replace(&mut self.squares, new_mine_index, Square::Mine);
            self.generate_numbers_around_mine(new_mine_pos);
        }
    }

    fn is_out_of_bounds(&self, pos: Position) -> bool {
        pos.x < 0 || 
        pos.y < 0 || 
        pos.x >= self.size.size_x || 
        pos.y >= self.size.size_y
    }

    fn generate_numbers_around_mine(&mut self, pos: Position) {
        for x in pos.x - 1..pos.x + 2 {
            for y in pos.y - 1..pos.y + 2 {
                let current_pos = Position { x, y };
                if self.is_out_of_bounds(current_pos) {
                    continue;
                }
                let current_pos = self.pos_to_index(current_pos);
                

                let current_square = self.squares[current_pos];
                match current_square {
                    Square::Nothing => replace(&mut self.squares, current_pos, Square::Num(1)),
                    Square::Num(i) => replace(&mut self.squares, current_pos, Square::Num(i + 1)),
                    _ => Square::Mine
                };
            }
        }
    }

    pub fn reveal_on_pos(&mut self, pos: Position, searched_positions: &mut Vec<usize>) -> bool { // if return value is true, the player lost
        let index = self.pos_to_index(pos);
        
        let reveal = self.squares[index];
        match reveal {
            Square::Nothing => {
                for x in pos.x - 1..pos.x + 2 {
                    for y in pos.y - 1..pos.y + 2 {
                        let current_pos = Position { x, y };
                        let current_index = self.pos_to_index(current_pos);
                        if current_pos == pos {
                            continue;
                        }
                        else if searched_positions.contains(&current_index) {
                            continue;
                        }
                        if self.is_out_of_bounds(current_pos) {
                            continue;
                        }
                        
                        searched_positions.push(current_index);
                        self.shown_squares[current_index] = Square::Revealed;
                        self.reveal_on_pos(current_pos, searched_positions);
                    }
                }
            },
            Square::Num(_) => self.shown_squares[index] = reveal,
            Square::Mine => return true,
            _ => ()
        };

        return false;
    }

    pub fn draw(&self) -> String {
        let mut output = string_repeat("==", self.size.size_y + 1) + "=\n";
        
        for x in 0..self.size.size_x {
            output += "|";
            for y in 0..self.size.size_y {
                let current_pos = self.pos_to_index(Position { x, y });
                let current_square = self.shown_squares[current_pos];
                if y == 0 {
                    output += " ";
                }

                match current_square {
                    Square::Nothing => output += "- ",
                    Square::Num(i) => output += &format!("{} ", i),
                    Square::Mine => output += "M ",
                    Square::Revealed => output += "  "
                };
            }
            output += "|\n";
        }
        output += &string_repeat("==", self.size.size_y + 1);
        output += "=";

        output
    }

    pub fn draw_ignore_visibility(&self) -> String {
        let mut output = String::new();
        
        for x in 0..self.size.size_x {
            for y in 0..self.size.size_y {
                let current_pos = self.pos_to_index(Position { x, y });
                let current_square = self.squares[current_pos];
                
                match current_square {
                    Square::Nothing => output += "- ",
                    Square::Num(i) => output += &format!("{} ", i),
                    Square::Mine => output += "M ",
                    Square::Revealed => ()
                };
            }
            output += "\n";
        }

        output
    }
}

fn print_to_center(s: String) {
    let (Width(width), Height(height)) = terminal_size().expect("Couldn't get terminal size");
    let lines = s.split("\n");

    if !(s.split("\n").count() > height as usize) {
        let padding_top = (height - s.split("\n").count() as u16) / 2;
        println!("{}", string_repeat("\n", padding_top.into()));    
    }

    for line in lines {
        if line.len() > width as usize {
            panic!("Your terminal is too small!");
        }
        let padding_left = (width - (line.len()) as u16) / 2;
        
        println!("{}{}", string_repeat(" ", padding_left as i32), line);
    }
    

}

fn clear_screen(term: &Term) {
    term.clear_screen().expect("Failed to clear screen");
}

fn input() -> String {
    let mut input = String::new();

    stdout().flush().expect("Failed to flush stdout!");

    stdin().read_line(&mut input).expect("Incorrect string!");
 
    if let Some('\n') = input.chars().next_back() {
        input.pop();
    }
    if let Some('\r') = input.chars().next_back() {
        input.pop();
    }

    input
}

fn main() {
    let term = Term::stdout();
    clear_screen(&term);

    term.set_title("Minesweeper");
    print!("Enter field size (x-y): ");

    let size = input();
    let size: Vec<&str> = size.split("-").collect();
    let size = Size {
        size_x: size[0].parse().expect("Bad height"), 
        size_y: size[1].parse().expect("Bad width") 
    };

    print!("Enter amount of mines: ");
    let mines: i32 = input().parse().expect("Bad number of mines");

    let mut field = Field::new(size, mines, 1);
    
    

    print_to_center(field.draw());
    
    
}