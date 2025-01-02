use rand::prelude::*;

const PLAYFIELD_WIDTH: usize = 10;
const PLAYFIELD_HEIGHT: usize = 16;

#[derive(Clone, Copy)]
pub enum TetrominoType {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl TetrominoType {
    pub fn rand(rng: &mut ThreadRng) -> TetrominoType {
        let types = [
            TetrominoType::I,
            TetrominoType::J,
            TetrominoType::L,
            TetrominoType::O,
            TetrominoType::S,
            TetrominoType::T,
            TetrominoType::Z,
        ];

        types[rng.gen_range(0..7) as usize]
    }
}

pub enum BoundIntersection {
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
    color: [u8; 3],
}

impl Tetromino {
    pub fn new(x: u8, y: u8, color: [u8; 3], shape: TetrominoType) -> Tetromino {
        Tetromino {
            x,
            y,
            orientation: 0,
            shape,
            color,
        }
    }

    pub fn shift(&mut self, x: i8, y: i8) {
        (self.x, self.y) = self.get_corrected_position(self.x as i8 + x, self.y as i8 + y);
    }

    pub fn rotate(&mut self) {
        self.orientation = match self.shape {
            TetrominoType::I => (self.orientation + 1) % 4,
            TetrominoType::J => (self.orientation + 1) % 4,
            TetrominoType::L => (self.orientation + 1) % 4,
            TetrominoType::S => (self.orientation + 1) % 4,
            TetrominoType::Z => (self.orientation + 1) % 4,
            TetrominoType::T => (self.orientation + 1) % 4,
            TetrominoType::O => self.orientation,
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
                } else if self.orientation == 1 {
                    return [
                        self.x,
                        self.y - 2,
                        self.x,
                        self.y - 1,
                        self.x,
                        self.y,
                        self.x,
                        self.y + 1,
                    ];
                } else if self.orientation == 2 {
                    return [
                        self.x - 1,
                        self.y + 1,
                        self.x,
                        self.y + 1,
                        self.x + 1,
                        self.y + 1,
                        self.x + 2,
                        self.y + 1,
                    ];
                }
                [
                    self.x - 1,
                    self.y - 2,
                    self.x - 1,
                    self.y - 1,
                    self.x - 1,
                    self.y,
                    self.x - 1,
                    self.y + 1,
                ]
            }

            TetrominoType::J => {
                if self.orientation == 0 {
                    return [
                        self.x + 1,
                        self.y,
                        self.x,
                        self.y,
                        self.x - 1,
                        self.y,
                        self.x - 1,
                        self.y - 1,
                    ];
                } else if self.orientation == 1 {
                    return [
                        self.x,
                        self.y,
                        self.x,
                        self.y - 1,
                        self.x,
                        self.y + 1,
                        self.x + 1,
                        self.y - 1,
                    ];
                } else if self.orientation == 2 {
                    return [
                        self.x,
                        self.y,
                        self.x + 1,
                        self.y,
                        self.x - 1,
                        self.y,
                        self.x + 1,
                        self.y + 1,
                    ];
                }

                [
                    self.x,
                    self.y,
                    self.x,
                    self.y - 1,
                    self.x,
                    self.y + 1,
                    self.x - 1,
                    self.y + 1,
                ]
            }

            TetrominoType::O => [
                self.x,
                self.y,
                self.x,
                self.y + 1,
                self.x + 1,
                self.y + 1,
                self.x + 1,
                self.y,
            ],

            TetrominoType::L => {
                if self.orientation == 0 {
                    return [
                        self.x + 1,
                        self.y,
                        self.x,
                        self.y,
                        self.x - 1,
                        self.y,
                        self.x + 1,
                        self.y - 1,
                    ];
                } else if self.orientation == 1 {
                    return [
                        self.x,
                        self.y,
                        self.x,
                        self.y - 1,
                        self.x,
                        self.y + 1,
                        self.x + 1,
                        self.y + 1,
                    ];
                } else if self.orientation == 2 {
                    return [
                        self.x,
                        self.y,
                        self.x + 1,
                        self.y,
                        self.x - 1,
                        self.y,
                        self.x - 1,
                        self.y + 1,
                    ];
                }

                [
                    self.x,
                    self.y,
                    self.x,
                    self.y - 1,
                    self.x,
                    self.y + 1,
                    self.x - 1,
                    self.y - 1,
                ]
            }

            TetrominoType::S => {
                if self.orientation == 0 {
                    return [
                        self.x,
                        self.y,
                        self.x,
                        self.y - 1,
                        self.x - 1,
                        self.y,
                        self.x + 1,
                        self.y - 1,
                    ];
                } else if self.orientation == 1 {
                    return [
                        self.x,
                        self.y,
                        self.x,
                        self.y - 1,
                        self.x + 1,
                        self.y,
                        self.x + 1,
                        self.y + 1,
                    ];
                } else if self.orientation == 2 {
                    return [
                        self.x,
                        self.y,
                        self.x + 1,
                        self.y,
                        self.x - 1,
                        self.y + 1,
                        self.x,
                        self.y + 1,
                    ];
                }

                [
                    self.x,
                    self.y,
                    self.x - 1,
                    self.y,
                    self.x,
                    self.y + 1,
                    self.x - 1,
                    self.y - 1,
                ]
            }

            TetrominoType::Z => {
                if self.orientation == 0 {
                    return [
                        self.x,
                        self.y,
                        self.x,
                        self.y - 1,
                        self.x + 1,
                        self.y,
                        self.x - 1,
                        self.y - 1,
                    ];
                } else if self.orientation == 1 {
                    return [
                        self.x,
                        self.y,
                        self.x,
                        self.y - 1,
                        self.x + 1,
                        self.y,
                        self.x + 1,
                        self.y + 1,
                    ];
                } else if self.orientation == 2 {
                    return [
                        self.x,
                        self.y,
                        self.x - 1,
                        self.y,
                        self.x,
                        self.y + 1,
                        self.x + 1,
                        self.y + 1,
                    ];
                }

                [
                    self.x,
                    self.y,
                    self.x - 1,
                    self.y,
                    self.x,
                    self.y - 1,
                    self.x - 1,
                    self.y + 1,
                ]
            }

            TetrominoType::T => {
                if self.orientation == 0 {
                    return [
                        self.x,
                        self.y,
                        self.x + 1,
                        self.y,
                        self.x - 1,
                        self.y,
                        self.x,
                        self.y - 1,
                    ];
                } else if self.orientation == 1 {
                    return [
                        self.x,
                        self.y,
                        self.x,
                        self.y - 1,
                        self.x,
                        self.y + 1,
                        self.x + 1,
                        self.y,
                    ];
                } else if self.orientation == 2 {
                    return [
                        self.x,
                        self.y,
                        self.x - 1,
                        self.y,
                        self.x + 1,
                        self.y,
                        self.x,
                        self.y + 1,
                    ];
                }

                [
                    self.x,
                    self.y,
                    self.x,
                    self.y - 1,
                    self.x,
                    self.y + 1,
                    self.x - 1,
                    self.y,
                ]
            }
        }
    }

    pub fn get_color(&self) -> [u8; 3] {
        self.color
    }

    pub fn get_position(&self) -> (i8, i8) {
        (self.x as i8, self.y as i8)
    }

    fn get_border_limits(&self) -> [i8; 4] {
        match self.shape {
            TetrominoType::I => {
                if self.orientation == 0 {
                    return [-1, 0, 2, 0];
                } else if self.orientation == 1 {
                    return [0, -2, 0, 1];
                } else if self.orientation == 2 {
                    return [-1, 1, 2, 1];
                }

                [-1, -2, -1, 1]
            }

            TetrominoType::J => {
                if self.orientation == 0 {
                    return [-1, -1, 1, 0];
                } else if self.orientation == 1 {
                    return [0, -1, 1, 1];
                } else if self.orientation == 2 {
                    return [-1, 0, 1, 1];
                }

                [-1, -1, 0, 1]
            }

            TetrominoType::L => {
                if self.orientation == 0 {
                    return [-1, -1, 1, 0];
                } else if self.orientation == 1 {
                    return [0, -1, 1, 1];
                } else if self.orientation == 2 {
                    return [-1, 0, 1, 1];
                }

                [-1, -1, 0, 1]
            }

            TetrominoType::S => {
                if self.orientation == 0 {
                    return [-1, -1, 1, 0];
                } else if self.orientation == 1 {
                    return [0, -1, 1, 1];
                } else if self.orientation == 2 {
                    return [-1, 0, 1, 1];
                }

                [-1, -1, 0, 1]
            }

            TetrominoType::Z => {
                if self.orientation == 0 {
                    return [-1, -1, 1, 0];
                } else if self.orientation == 1 {
                    return [0, -1, 1, 1];
                } else if self.orientation == 2 {
                    return [-1, 0, 1, 1];
                }

                [-1, -1, 0, 1]
            }

            TetrominoType::T => {
                if self.orientation == 0 {
                    return [-1, -1, 1, 0];
                } else if self.orientation == 1 {
                    return [0, -1, 1, 1];
                } else if self.orientation == 2 {
                    return [-1, 0, 1, 1];
                }

                [-1, -1, 0, 1]
            }

            TetrominoType::O => [0, -1, 1, 1],
        }
    }

    pub fn get_bound_intersection(&self, x: i8, y: i8) -> BoundIntersection {
        let limits = self.get_border_limits();

        if x + limits[0] < 0 {
            return BoundIntersection::LEFT;
        } else if x + limits[2] > (PLAYFIELD_WIDTH - 1) as i8 {
            return BoundIntersection::RIGHT;
        }

        if y + limits[1] < 0 {
            return BoundIntersection::TOP;
        } else if y + limits[3] > (PLAYFIELD_HEIGHT - 1) as i8 {
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
