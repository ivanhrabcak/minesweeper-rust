use clap::{App, Arg, ArgMatches};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use terminal_size::{Height, Width, terminal_size};

#[cfg(unix)]
use os::unix::prelude::AsRawFd;

use std::{io::{Write, stdin, stdout}, mem, str::FromStr, time::{Instant}};
use console::Term;
use device_query::{DeviceState, Keycode};


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
    Revealed,
    Marked
}

#[derive(Debug, Clone)]
struct Field {
    pub seed: u64,
    squares: Vec<Square>,
    shown_squares: Vec<Square>,
    size: Size,
    time: Instant,
    mines: i32
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
        let mut field = Field { squares: Vec::new(), shown_squares: Vec::new(), size, seed, mines, time: Instant::now() };
        field.init();
        field
    }
    
    pub fn pos_to_index(&self, pos: Position) -> usize {
        (pos.x * self.size.size_x + pos.y) as usize 
    }

    fn init(&mut self) {
        let field_size = self.size.size_x * self.size.size_y;
    
        for _ in 0..field_size {
            self.squares.push(Square::Nothing);
        }

        self.shown_squares = self.squares.clone();
        
        let mut rng: Pcg64 = match self.seed {
            0 => Pcg64::from_entropy(),
            i => Pcg64::seed_from_u64(i)
        };
        
        for _ in 0..self.mines {
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

    pub fn reveal_on_pos(&mut self, pos: Position, searched_positions: &mut Vec<usize>) -> bool { // if the return value is true, the player lost
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
            Square::Mine => {
                self.shown_squares[index] = reveal;
                return true;
            },
            _ => ()
        };

        return false;
    }

    pub fn player_won(&self) -> bool {
        if self.count_marked() != self.mines {
            return false;
        } 

        for x in 0..self.size.size_x {
            for y in 0..self.size.size_y {
                let current_pos = Position { x, y };
                let current_index = self.pos_to_index(current_pos);

                let current_shown_square = self.shown_squares[current_index];
                if current_shown_square != Square::Marked {
                    continue;
                }

                let current_square = self.squares[current_index];
                if current_square != Square::Mine {
                    return false;
                }
            }
        }

        true
    }

    fn count_marked(&self) -> i32 {
        let mut counter = 0;
        for square in self.shown_squares.iter() {
            if square == &Square::Marked {
                counter += 1;
            }
        }

        counter
    }
    
    pub fn status_bar(&self) -> String {
        let mut output = String::new();
        
        output += &self.count_marked().to_string();
        output += "/";
        output += &self.mines.to_string();

        output += " ";
        output += &self.time.elapsed().as_secs().to_string();

        output
    }

    pub fn toggle_mark(&mut self, pos: Position) {
        let index = self.pos_to_index(pos);
        match self.shown_squares[index] {
            Square::Marked => self.shown_squares[index] = Square::Nothing,
            Square::Nothing => self.shown_squares[index] = Square::Marked,
            _ => ()
        };
    }

    pub fn draw(&self, position_on_field: Position) -> String {
        let mut output = string_repeat("==", self.size.size_y + 1) + "=\n";

        for x in 0..self.size.size_x {
            output += "|";
            for y in 0..self.size.size_y {
                let current_pos = self.pos_to_index(Position { x, y });
                let current_square = self.shown_squares[current_pos];
                if y == 0 {
                    output += " ";
                }

                if (Position { x, y }) == position_on_field {
                    output.pop();
                    output += "["
                }

                match current_square {
                    Square::Nothing => output += "- ",
                    Square::Num(i) => output += &format!("{} ", i),
                    Square::Mine => output += "M ",
                    Square::Revealed => output += "  ",
                    Square::Marked => output += "* "
                }
                
                if (Position { x, y }) == position_on_field {
                    output.pop();
                    output += "]";
                }
            }
            output += "|\n";
        }
        output += &string_repeat("==", self.size.size_y + 1);
        output += "=";

        output
    }
}

fn print_to_center(s: String, horizontal: bool, vertical: bool) {
    let (Width(width), Height(height)) = terminal_size().expect("Couldn't get terminal size");
    let lines = s.split("\n");

    if vertical && !(s.split("\n").count() > height as usize) {
        let padding_top = (height - s.split("\n").count() as u16) / 2;
        println!("{}", string_repeat("\n", padding_top.into()));    
    }

    if !horizontal {
        println!("{}", s);
        return;
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

fn get_field<T: FromStr>(matches: &ArgMatches, name: &str, default: T) -> T {
    match matches.value_of(name) {
        None => {
            default
        },
        Some(v) => {
            match v.parse() {
                Ok(l) => l,
                Err(_) => {
                    default
                }
            }
        }
    }
}

fn draw(term: &Term, field: &Field, show_hint: bool, current_position: Position) {
    clear_screen(term);

    let mut drawn_field = field.status_bar() + "\n";
    drawn_field += &(field.draw(current_position) + "\n");

    if show_hint {
        drawn_field += "Use the arrow keys to move around the field, SPACE to reveal an M to mark.";
    }
    
    print_to_center(drawn_field, true, true);
}

fn main() {
    let matches = App::new("Minesweeper")
                                    .arg(Arg::with_name("field_height")
                                            .short("h")
                                            .long("height")
                                            .help("Height of the field")
                                            .takes_value(true))
                                    .arg(Arg::with_name("field_width")
                                            .short("w")
                                            .long("width")
                                            .help("Width of the field")
                                            .takes_value(true))
                                    .arg(Arg::with_name("mines")
                                            .short("m")
                                            .long("mines")
                                            .help("Number of mines")
                                            .takes_value(true))
                                    .arg(Arg::with_name("seed")
                                            .short("s")
                                            .long("seed")
                                            .help("Seed for mine generation")
                                            .takes_value(true))
                                    .get_matches();
    let height = get_field(&matches, "height", 10);
    let width = get_field(&matches, "width", 10);
    let size = Size { size_x: width, size_y: height };

    let mines = get_field(&matches, "mines", 10);
    
    let seed: u64 = matches.value_of("seed").unwrap_or("0")
                .parse().unwrap();
    
    let mut field = Field::new(size, mines, seed);
    
    let term = Term::stdout();
    
    #[cfg(unix)]
    term.as_raw_fd();

    let mut start = Instant::now();
    let device_state = DeviceState::new();

    let mut show_hint = true;

    clear_screen(&term);
    
    let mut current_position = Position { x: field.size.size_x / 2, y: field.size.size_y / 2 };
    let mut drawn_field = field.status_bar() + "\n";
    drawn_field += &(field.draw(current_position) + "\n");
    drawn_field += "Use the arrow keys to move around the field, SPACE to reveal an M to mark.";

    
    print_to_center(drawn_field, true, true);
    
    
    let mut keys_held_previous_iteration: Vec<Keycode> = Vec::new();
    while !field.player_won() {
        let keys: Vec<Keycode> = device_state.query_keymap();
        let pressed_keys: Vec<Keycode> = keys.clone().into_iter().filter(|x| !keys_held_previous_iteration.contains(x)).collect();
        keys_held_previous_iteration = keys.clone();

        if pressed_keys.contains(&Keycode::Up) {
            show_hint = false;
            current_position.x -= 1;

            if current_position.x < 0 {
                current_position.x = 0;
            }
            draw(&term, &field, show_hint, current_position);

        }
        if pressed_keys.contains(&Keycode::Down) {
            show_hint = false;
            current_position.x += 1;

            if current_position.x == field.size.size_x {
                current_position.x = field.size.size_x - 1;
            }
            draw(&term, &field, show_hint, current_position);

        }
        if pressed_keys.contains(&Keycode::Left) {
            show_hint = false;
            current_position.y -= 1;

            if current_position.y < 0 {
                current_position.y = 0;
            }
            draw(&term, &field, show_hint, current_position);

        }
        if pressed_keys.contains(&Keycode::Right) {
            show_hint = false;
            current_position.y += 1;

            if current_position.y == field.size.size_y {
                current_position.y = field.size.size_y - 1;
            }
            draw(&term, &field, show_hint, current_position);
        }
        if pressed_keys.contains(&Keycode::M) {
            show_hint = false;
            field.toggle_mark(current_position);
            draw(&term, &field, show_hint, current_position);
        }
        if pressed_keys.contains(&Keycode::Space) {
            show_hint = false;
            let player_lost = field.reveal_on_pos(current_position, &mut Vec::new());
            if !player_lost {
                draw(&term, &field, show_hint, current_position);
            }
            else {
                clear_screen(&term);

                let mut drawn_field = field.status_bar() + "\n";
                drawn_field += &(field.draw(current_position) + "\n");
                drawn_field += "BOOM! You've lost!";
                
                print_to_center(drawn_field, true, true);


                break;
            }
            
        }

        if start.elapsed().as_millis() >= 1000 {
            draw(&term, &field, show_hint, current_position);

            start = Instant::now();
        }
    }

    if field.player_won() {
        clear_screen(&term);

        let mut drawn_field = field.status_bar() + "\n";
        drawn_field += &(field.draw(current_position) + "\n");
        drawn_field += "You win!";
        
        print_to_center(drawn_field, true, true);
    }
}