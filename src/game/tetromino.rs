pub enum TetrominoType {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

pub struct Tetromino {
    shape: TetrominoType,
    orientation: u8,
    x: u8,
    y: u8,
}

impl Tetromino {
    pub fn new(x: u8, y: u8, shape: TetrominoType) -> Tetromino {
        Tetromino {
            x,
            y,
            orientation: 0,
            shape,
        }
    }

    pub fn shift(&mut self, x: u8, y: u8) {
        self.x += x;
        self.y += y;
    }

    pub fn get_blocks(&self) -> [u8; 8] {
        match self.shape {
            TetrominoType::I => {
                if self.orientation == 0 {
                    return [
                        self.x,
                        self.y,
                        self.x + 1,
                        self.y,
                        self.x + 2,
                        self.y,
                        self.x + 3,
                        self.y,
                    ];
                }

                [
                    self.x,
                    self.y,
                    self.x,
                    self.y + 1,
                    self.x,
                    self.y + 2,
                    self.x,
                    self.y + 3,
                ]
            }

            _ => [0; 8],
        }
    }
}
