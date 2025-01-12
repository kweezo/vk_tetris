use rand::prelude::*;

use crate::types::*;

#[derive(Clone, Copy)]
pub enum TetrominoShape {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl TetrominoShape {
    pub fn rand(rng: &mut ThreadRng) -> TetrominoShape {
        let types = [
            TetrominoShape::I,
            TetrominoShape::J,
            TetrominoShape::L,
            TetrominoShape::O,
            TetrominoShape::S,
            TetrominoShape::T,
            TetrominoShape::Z,
        ];

        types[rng.gen_range(0..7) as usize]
    }
}

#[derive(Clone, Copy)]
pub enum Orientation{
    ZERO = 0,
    RIGHT = 1,
    TWO = 2,
    LEFT = 3
}

pub struct Tetromino {
    shape: TetrominoShape,

    orientation: Orientation,

    pos: (i8, i8),
    color: [u8; 3],

    offset_index: usize
}

impl Tetromino {
    pub fn new(pos: (i8, i8), color: [u8; 3], shape: TetrominoShape) -> Tetromino {
        Tetromino {
            pos,
            orientation: Orientation::ZERO,
            shape,
            color,
            offset_index: 0
        }
    }


    fn get_raw_data(&self, offset_index: usize) -> ([(i8, i8); 4], (i8, i8)){
        let mut shape_data: [(i8, i8); 4] = match self.shape{
            TetrominoShape::I => [(1, 0), (0, 0), (-1, 0), (-2, 0)],
            TetrominoShape::J => [(1, -1), (1, 0), (0, 0), (-1, 0)],
            TetrominoShape::L => [(-1, -1), (1, 0), (0, 0), (-1, 0)],
            TetrominoShape::O => [(0, 0), (0, -1), (-1, 0), (-1, -1)],
            TetrominoShape::S => [(1, 0), (0 ,0), (0, -1), (-1, -1)],
            TetrominoShape::T => [(1, 0), (0, 0), (0, -1), (-1, 0)],
            TetrominoShape::Z => [(1, -1), (0, -1), (0, 0), (-1, 0)]
        };

        let offsets_i: [[(i8, i8); 5]; 4] = [ 
 	    [(0, 0), (1, 0), (-2, 0), (1, 0), (-2, 0)],
 	    [(1, 0), ( 0, 0), ( 0, 0), (0, 1), ( 0,-2)],
 	    [(1, 1), (-1, 1), (2, 1), (-1, 0), (2, 0)],
 	    [(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)]];
        
        let offsets_o: [(i8, i8); 4] = [(0, 0), (0, -1), (-1, -1), (-1, 0)];

        let offsets_other: [[(i8, i8); 5]; 4]= [
        [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
        [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
     	[(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
 	    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)]];



        let offset = match self.shape{
            TetrominoShape::I => {
                offsets_i[self.orientation as usize][offset_index]
            },
            TetrominoShape::O => {
                offsets_o[self.orientation as usize]
            },
            _ => {
                offsets_other[self.orientation as usize][offset_index]
            }
        };


        for block in shape_data.iter_mut(){
            match self.orientation {
                Orientation::ZERO => (),

                Orientation::RIGHT => {
                    (block.0, block.1) = (-block.1, block.0);
                },
                
                Orientation::TWO => {
                    (block.0, block.1) = (-block.0, -block.1);
                },

                Orientation::LEFT => {
                    (block.0, block.1) = (block.1, -block.0);
                },

            }

            block.0 += offset.0;
            block.1 += offset.1;

        }

        (shape_data, offset)
    }

     fn is_in_bounds(data: &[(i8, i8); 4]) -> (i8, i8){
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (data[0].0, data[0].1, data[0].0, data[0].1);

        for block in data.iter(){
            min_x = min_x.min(block.0);
            min_y = min_y.min(block.1);
            max_x = max_x.max(block.0);
            max_y = max_y.max(block.1);
        }

        if min_x < 0 {
            return (-min_x, 0);
        } else if max_x >= PLAYFIELD_WIDTH as i8{
            return (PLAYFIELD_WIDTH as i8 - max_x - 1, 0);
        } else if min_y < 0 {
            return (0, -min_y);
        } else if max_y >= PLAYFIELD_HEIGHT as i8{
            return (0, PLAYFIELD_HEIGHT as i8 - max_y - 1);
        }

        (0, 0)
    }

    fn is_valid(data: &[(i8, i8); 4], grid: &Grid) -> bool{

        if Tetromino::is_in_bounds(data) != (0, 0) {
            return false;
        }

        for block in data.iter(){
            if grid[block.1 as usize][block.0 as usize] != [0; 3] {
                return false;
            }
        }

        true
    }

    fn correct_position(&mut self, grid: &Grid) -> bool{
        for i in 0..5 {
            let (mut data, offset) = Tetromino::get_raw_data(&self, i);

            for block in data.iter_mut() {
                block.0 += self.pos.0 + offset.0;
                block.1 += self.pos.1 + offset.1;
            }

            if Tetromino::is_valid(&data, grid){
                self.offset_index = i;

                return true;
            }
        }

        false
    }

    pub fn translate(&mut self, vec: (i8, i8), grid: &Grid) {
        let previous_pos = self.pos;

        self.pos.0 += vec.0;
        self.pos.1 += vec.1;

        let (mut data, _) = Tetromino::get_raw_data(&self, self.offset_index);
        for block in data.iter_mut() {
            block.0 += self.pos.0;
            block.1 += self.pos.1;
        }

        let bounds_offset = Tetromino::is_in_bounds(&data);

        self.pos.0 += bounds_offset.0;
        self.pos.1 += bounds_offset.1;

        
        if !Tetromino::is_valid(&data, grid){
            self.pos = previous_pos;
        }
    }

    pub fn rotate(&mut self, dir: Orientation, grid: &Grid) {
        let previous_orientation = self.orientation;

        match self.shape {
            TetrominoShape::O => (),
            _ => {
                let curr = (self.orientation as u32 + dir as u32) % 4;

                self.orientation = match curr{
                    0 => Orientation::ZERO,
                    1 => Orientation::RIGHT,
                    2 => Orientation::TWO,
                    _ => Orientation::LEFT
                };
            }
        }

        if !self.correct_position(grid){
            self.orientation = previous_orientation;
        }
    }

    pub fn get_data(&self) -> [u8; 8]{

        let (data, _) = Tetromino::get_raw_data(&self,self.offset_index);

        let mut data_unwrapped = [0u8; 8];

        for i in 0..data.len() {
            data_unwrapped[i*2] = (data[i].0 + self.pos.0) as u8;
            data_unwrapped[i*2+1] = (data[i].1 + self.pos.1) as u8;
        }

        data_unwrapped 
    }
 

    pub fn is_grounded(&self, grid: &Grid) -> bool{
        let (mut data, _) = Tetromino::get_raw_data(&self, self.offset_index);

        for block in data.iter_mut() {
            block.0 += self.pos.0;
            block.1 += self.pos.1 + 1; //TODO NO WORKY WRKY
        }

        if Tetromino::is_valid(&data, grid) {
            return false;
        }

        true
    }

    pub fn get_color(&self) -> [u8; 3] {
        self.color
    }
}
