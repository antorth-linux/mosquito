const MIN_REGION_SIZE: Rectangle = Rectangle { w: 20, h: 20 };

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ErrorKind {
    UnknownRegion,
    UnknownWorkspace,
    UnknownMonitor,
    InvalidRegion,
}

pub type Result<T> = std::result::Result<T, ErrorKind>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rectangle {
    pub w: u64,
    pub h: u64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Region {
    pub size: Rectangle,
    pub pos: Position,
    pub float: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Workspace {
    pub size: Rectangle,
    pub regions: Vec<Region>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Monitor {
    pub size: Rectangle,
    pub pos: Position,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Manager {
    pub workspaces: Vec<Workspace>,
    pub monitors: Vec<Monitor>,
}

impl Rectangle {
    pub fn new(width: u64, height: u64) -> Self {
        Self {
            w: width,
            h: height,
        }
    }
}

impl Position {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}

impl Region {
    pub fn new(size: Rectangle, pos: Position, float: bool) -> Self {
        Self { size, pos, float }
    }

    pub fn area(&self) -> u64 {
        self.size.w * self.size.h
    }

    pub fn top(&self) -> i64 {
        self.pos.y
    }

    pub fn bottom(&self) -> i64 {
        self.pos.y + self.size.h as i64
    }

    pub fn left(&self) -> i64 {
        self.pos.x
    }

    pub fn right(&self) -> i64 {
        self.pos.x + self.size.w as i64
    }

    pub fn set_top(&mut self, new: i64) -> Result<&mut Self> {
        if new > self.bottom() - MIN_REGION_SIZE.h as i64 {
            return Err(ErrorKind::InvalidRegion);
        }

        self.size.h = (self.bottom() - new) as u64;
        self.pos.y = new;

        Ok(self)
    }

    pub fn set_bottom(&mut self, new: i64) -> Result<&mut Self> {
        if new < self.top() + MIN_REGION_SIZE.h as i64 {
            return Err(ErrorKind::InvalidRegion);
        }

        self.size.h = (new - self.top()) as u64;

        Ok(self)
    }

    pub fn set_left(&mut self, new: i64) -> Result<&mut Self> {
        if new > self.right() - MIN_REGION_SIZE.w as i64 {
            return Err(ErrorKind::InvalidRegion);
        }

        self.size.w = (self.right() - new) as u64;
        self.pos.x = new;

        Ok(self)
    }

    pub fn set_right(&mut self, new: i64) -> Result<&mut Self> {
        if new < self.left() + MIN_REGION_SIZE.w as i64 {
            return Err(ErrorKind::InvalidRegion);
        }

        self.size.w = (new - self.left()) as u64;

        Ok(self)
    }
}

impl Workspace {
    pub fn new(size: Rectangle) -> Self {
        let mut this = Self {
            size,
            regions: Vec::new(),
        };

        this.regions
            .extend([Region::new(size, Position::new(0, 0), false)]);

        this
    }

    pub fn create_region(&mut self, sibling: &mut Region, direction: &Direction) -> &Region {
        let mut region = *sibling;

        match *direction {
            Direction::Up => {
                region.size.h /= 2;
                sibling.size.h /= 2;
                sibling.pos.y += region.size.h as i64;
            }
            Direction::Down => {
                region.size.h /= 2;
                sibling.size.h /= 2;
                region.pos.y += sibling.size.h as i64;
            }
            Direction::Left => {
                region.size.w /= 2;
                sibling.size.w /= 2;
                sibling.pos.x += region.size.w as i64;
            }
            Direction::Right => {
                region.size.w /= 2;
                sibling.size.w /= 2;
                region.pos.x += sibling.size.w as i64;
            }
        }

        self.regions.extend([region]);

        self.regions.last().unwrap()
    }

    pub fn adjacent_regions(&self, region: &Region, direction: &Direction) -> Vec<&Region> {
        let mut adjacent = Vec::new();

        for sibling in self.regions.iter() {
            if match *direction {
                Direction::Up => (0..1).contains(&(region.top() - sibling.bottom())),
                Direction::Down => (-1..0).contains(&(region.bottom() - sibling.top())),
                Direction::Left => (0..1).contains(&(region.left() - sibling.right())),
                Direction::Right => (-1..0).contains(&(region.right() - sibling.left())),
            } {
                adjacent.extend([sibling])
            }
        }

        adjacent
    }

    // pub fn move_region(&mut self, region: &Region, direction: &Direction) {
    //     match direction {
    //         Direction::Up => {}
    //         Direction::Down => {}
    //         Direction::Left => {}
    //         Direction::Right => {}
    //     }
    // }

    // pub fn resize(&mut self, new: Rectangle) -> Result<()> {
    //     let scale_w = new.w as f64 / self.size.w as f64;
    //     let scale_h = new.h as f64 / self.size.h as f64;

    //     for mut region in &mut self.regions {
    //         region.pos.x = (region.pos.x as f64 * scale_w) as i64;
    //         region.pos.y = (region.pos.y as f64 * scale_h) as i64;
    //         region.size.w = (region.size.w as f64 * scale_w) as u64;
    //         region.size.h = (region.size.h as f64 * scale_h) as u64;
    //     }

    //     self.size = new;

    //     Ok(())
    // }
}

// impl Monitor {
//     pub fn new<I>(size: Rectangle, pos: Position) -> Self {
//         Self { size, pos }
//     }
// }

// impl Manager {
//     pub fn create_workspace(&mut self, monitor: &Monitor) -> Result<&Workspace> {
//         if !self.monitors.contains(monitor) {
//             return Err(ErrorKind::UnknownMonitor);
//         }

//         self.workspaces.extend([Workspace::new(monitor.size)]);

//         Ok(self.workspaces.last().unwrap())
//     }

//     pub fn remove_workspace(&mut self, workspace: &Workspace) -> Result<()> {
//         self.workspaces.remove(self.workspace_index(workspace)?);

//         Ok(())
//     }

//     pub fn move_workspace(&mut self, workspace: &mut Workspace, monitor: &Monitor) -> Result<()> {
//         workspace.resize(monitor.size)
//     }

//     fn workspace_index(&self, workspace: &Workspace) -> Result<usize> {
//         self.workspaces
//             .iter()
//             .position(|x| x == workspace)
//             .ok_or(ErrorKind::UnknownWorkspace)
//     }
// }
