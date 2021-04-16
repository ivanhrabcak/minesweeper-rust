use rand::{Rng, prelude::ThreadRng};
use std::mem;

#[derive(Debug)]
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
    Nothing
}

#[derive(Debug, Clone)]
struct Field {
    squares: Vec<Square>,
    size: Size
}

fn replace<T>(vec: &mut Vec<T>, i: usize, e: T) -> T {
    mem::replace(&mut vec[i], e)
}

impl Field {
    pub fn new(size: Size, mines: i32) -> Self {
        

        let mut field = Field { squares: Vec::new(), size };
        field.init(mines);
        field
    }

    fn init(&mut self, mines: i32) {
        let field_size = (self.size.size_x * self.size.size_y) as usize;
    
        for _ in 0..field_size {
            self.squares.push(Square::Nothing);
        }

        let mut rng  = rand::thread_rng();
        for _ in 0..mines {
            let mut new_mine_pos: Position = Position { x: rng.gen_range(0..self.size.size_x), y: rng.gen_range(0..self.size.size_y) };
            
            let mut new_mine_index = (new_mine_pos.x * self.size.size_x + new_mine_pos.y) as usize; 
            while self.squares.get(new_mine_index).unwrap() != &Square::Nothing {
                new_mine_pos = Position { x: rng.gen_range(0..self.size.size_x), y: rng.gen_range(0..self.size.size_y) };
                new_mine_index = (new_mine_pos.x * self.size.size_x + new_mine_pos.y) as usize; 
            }
            
            replace(&mut self.squares, new_mine_index, Square::Mine);
            self.generate_numbers_around_mine(new_mine_pos);
        }

        // let mine_pos = Position { x: self.size.size_x / 2, y: self.size.size_y / 2 };
        // replace(&mut self.squares, (mine_pos.x * self.size.size_x + mine_pos.y) as usize, Square::Mine);
        // self.generate_numbers_around_mine(mine_pos);

        // let mine_pos = Position { x: self.size.size_x / 2 - 2, y: self.size.size_y / 2 };
        // replace(&mut self.squares, (mine_pos.x * self.size.size_x + mine_pos.y) as usize, Square::Mine);
        // self.generate_numbers_around_mine(mine_pos);

        // let mine_pos = Position { x: 0, y: 0 };
        // replace(&mut self.squares, (mine_pos.x * self.size.size_x + mine_pos.y) as usize, Square::Mine);
        // self.generate_numbers_around_mine(mine_pos);
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
                let current_pos = (self.size.size_x * x + y) as usize;
                

                let current_square = self.squares[current_pos];
                match current_square {
                    Square::Nothing => replace(&mut self.squares, current_pos, Square::Num(1)),
                    Square::Num(i) => replace(&mut self.squares, current_pos, Square::Num(i + 1)),
                    _ => Square::Mine
                };
            }
        }
    }

    pub fn draw(&self) -> String {
        let mut output = String::new();
        
        for x in 0..self.size.size_x {
            for y in 0..self.size.size_y {
                let current_pos = (x * self.size.size_x + y) as usize;
                let current_square = self.squares[current_pos];
                
                match current_square {
                    Square::Nothing => output += "- ",
                    Square::Num(i) => output += &format!("{} ", i),
                    Square::Mine => output += "M "
                };
            }
            output += "\n";
        }

        output
    }
}

fn main() {
    let field = Field::new(Size { size_x: 10, size_y: 10 }, 10);
    println!("{}", field.draw());
}
