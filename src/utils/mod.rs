pub mod coords;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Direction {
    #[default]
    Left = -1,
    Right = 1,
}

impl Direction {
    pub fn get_opposite(&self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
    pub fn as_int(self) -> i32 {
        self as i32
    }
}
