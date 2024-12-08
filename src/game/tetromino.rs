
enum TetrominoType{
    I,
    J,
    L,
    O,
    S,
    T,
    Z
}

pub struct Tetromino{
    shape: TetrominoType,
    x: u8,
    y: u8
}

impl Tetromino{
    pub fn shift(&mut self, x: u8, y: u8){
        self.x += x;
        self.y += y;
    }

    pub fn get_blocks(&self) -> [u8; 4]{
        [self.x, self.y, self.x, self.y]
    }
}