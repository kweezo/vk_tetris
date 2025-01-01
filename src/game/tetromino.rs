const PLAYFIELD_WIDTH: usize = 10;
const PLAYFIELD_HEIGHT: usize = 16;

pub enum TetrominoType {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

enum BoundIntersection {
    LEFT,
    RIGHT,
    TOP,
    BOTTOM,
    NONE,
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

    pub fn shift(&mut self, x: i8, y: i8) {
        (self.x, self.y) = self.get_corrected_position(self.x as i8 + x, self.y as i8 + y);
    }

    pub fn rotate(&mut self) {
        self.orientation = match self.shape {
            TetrominoType::I => (self.orientation + 1) % 2,
            _ => 0,
        };

        (self.x, self.y) = self.get_corrected_position(self.x as i8, self.y as i8);
    }

    pub fn get_blocks(&self) -> [u8; 8] {
        match self.shape {
            TetrominoType::I => {
                if self.orientation == 0 {
                    return [
                        self.x - 1,
                        self.y,
                        self.x,
                        self.y,
                        self.x + 1,
                        self.y,
                        self.x + 2,
                        self.y,
                    ];
                }

                [
                    self.x,
                    self.y - 2,
                    self.x,
                    self.y - 1,
                    self.x,
                    self.y,
                    self.x,
                    self.y + 1,
                ]
            }

            _ => [0; 8],
        }
    }

    fn get_border_limits(&self) -> [i8; 4] {
        match self.shape {
            TetrominoType::I => {
                if self.orientation == 0 {
                    return [-1, 0, 2, 0];
                }

                [0, -2, 0, 1]
            }

            _ => [0; 4],
        }
    }

    fn get_bound_intersection(&self, x: i8, y: i8) -> BoundIntersection {
        let limits = self.get_border_limits();

        if x + limits[0] < 0 {
            return BoundIntersection::LEFT;
        } else if x + limits[2] > PLAYFIELD_WIDTH as i8 {
            return BoundIntersection::RIGHT;
        }

        if y + limits[1] < 0 {
            return BoundIntersection::TOP;
        } else if y + limits[3] > PLAYFIELD_HEIGHT as i8 {
            return BoundIntersection::BOTTOM;
        }

        BoundIntersection::NONE
    }

    fn get_corrected_position(&self, x: i8, y: i8) -> (u8, u8) {
        let mut corrected_x = x;
        let mut corrected_y = y;

        loop {
            match self.get_bound_intersection(corrected_x, corrected_y) {
                BoundIntersection::LEFT => corrected_x += 1,
                BoundIntersection::RIGHT => corrected_x -= 1,
                BoundIntersection::BOTTOM => corrected_y -= 1,
                BoundIntersection::TOP => corrected_y += 1,
                BoundIntersection::NONE => break,
            };
        }

        (corrected_x as u8, corrected_y as u8)
    }
}
