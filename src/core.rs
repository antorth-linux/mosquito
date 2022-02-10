const MIN_REGION_SIZE: Rectangle = Rectangle { w: 20, h: 20 };

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ErrorKind {
    UnknownRegion,
    UnknownWorkspace,
    UnknownMonitor,
    InvalidRegion,
    NoAdjacentRegions,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Resize {
    Top(i64),
    Bottom(i64),
    Left(i64),
    Right(i64),
    TopLeft(i64, i64),
    TopRight(i64, i64),
    BottomLeft(i64, i64),
    BottomRight(i64, i64),
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

    /// Create a new region using half the width or height of a sibling region.
    /// The `direction` specifies which edge of the sibling will be moved to make space for the new region.
    /// The sibling will be the larger region in the event that the halved dimension of the sibling region is an odd number.
    pub fn create_region(&mut self, sibling: &mut Region, direction: &Direction) -> usize {
        let mut region = *sibling;

        match *direction {
            Direction::Up => {
                region.size.h /= 2;
                sibling.size.h = sibling.size.h / 2 + sibling.size.h % 2;
                sibling.pos.y += region.size.h as i64;
            }
            Direction::Down => {
                region.size.h /= 2;
                sibling.size.h = sibling.size.h / 2 + sibling.size.h % 2;
                region.pos.y += sibling.size.h as i64;
            }
            Direction::Left => {
                region.size.w /= 2;
                sibling.size.w = sibling.size.w / 2 + sibling.size.w % 2;
                sibling.pos.x += region.size.w as i64;
            }
            Direction::Right => {
                region.size.w /= 2;
                sibling.size.w = sibling.size.w / 2 + sibling.size.w % 2;
                region.pos.x += sibling.size.w as i64;
            }
        }

        self.regions.extend([region]);

        self.regions.len() - 1
    }

    pub fn shared_edge_regions(&self, region: &Region, direction: &Direction) -> Vec<usize> {
        self.regions
            .iter()
            .enumerate()
            .filter_map(|(index, sibling)| {
                if match *direction {
                    Direction::Up => region.top() - sibling.bottom() == 1,
                    Direction::Down => region.bottom() - sibling.top() == 1,
                    Direction::Left => region.left() - sibling.right() == 1,
                    Direction::Right => region.right() - sibling.left() == 1,
                } {
                    Some(index)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn adjacent_regions(&self, region: &Region, direction: &Direction) -> Vec<usize> {
        self.shared_edge_regions(region, direction)
            .into_iter()
            .filter(|index| {
                let sibling = self.regions.get(*index).unwrap();

                match *direction {
                    Direction::Up | Direction::Down => {
                        (sibling.left() >= region.left() && sibling.left() <= region.right())
                            || (sibling.right() <= region.right()
                                && sibling.right() >= region.left())
                    }
                    Direction::Left | Direction::Right => {
                        (sibling.top() >= region.top() && sibling.top() <= region.bottom())
                            || (sibling.bottom() <= region.bottom()
                                && sibling.bottom() >= region.top())
                    }
                }
            })
            .collect()
    }

    /// Find the adjacent region with the largest overlap on the edge corresponding with `direction`.
    /// In the event that there are no regions touching the edge of the subject, `None` will be returned.
    /// This currently does not take into account that several sibling regions may have the same overlap;
    /// which sibling will be returned is undefined, however it will most likely be the sibling that was
    /// created the latest as it is going to have a higher index in the internal iterable.
    pub fn major_adjacent_region(&self, region: &Region, direction: &Direction) -> Option<usize> {
        self.adjacent_regions(region, direction)
            .into_iter()
            .map(|index| {
                let sibling = self.regions.get(index).unwrap();

                (
                    index,
                    match *direction {
                        Direction::Up | Direction::Down => i64::abs(
                            i64::max(sibling.left(), region.left())
                                - i64::min(sibling.right(), region.right()),
                        ) as u64,
                        Direction::Left | Direction::Right => i64::abs(
                            i64::max(sibling.top(), region.top())
                                - i64::min(sibling.bottom(), region.bottom()),
                        ) as u64,
                    },
                )
            })
            .max_by_key(|x| x.1)
            .map(|x| x.0)
    }

    pub fn resize_region(&mut self, region: &mut Region, resize: &Resize) -> Result<()> {
        match *resize {
            Resize::Top(top) => {
                region.set_top(region.top() + top)?;

                for index in self.adjacent_regions(region, &Direction::Up) {
                    let sibling = self.regions.get_mut(index).unwrap();

                    sibling.set_bottom(sibling.bottom() - top)?;
                }
            }
            Resize::Bottom(bottom) => {
                region.set_bottom(region.bottom() + bottom)?;

                for index in self.adjacent_regions(region, &Direction::Down) {
                    let sibling = self.regions.get_mut(index).unwrap();

                    sibling.set_top(sibling.top() - bottom)?;
                }
            }
            Resize::Left(left) => {
                region.set_left(region.left() + left)?;

                for index in self.adjacent_regions(region, &Direction::Left) {
                    let sibling = self.regions.get_mut(index).unwrap();

                    sibling.set_right(sibling.right() - left)?;
                }
            }
            Resize::Right(right) => {
                region.set_right(region.right() + right)?;

                for index in self.adjacent_regions(region, &Direction::Right) {
                    let sibling = self.regions.get_mut(index).unwrap();

                    sibling.set_left(sibling.left() - right)?;
                }
            }
            Resize::TopLeft(top, left) => {
                self.resize_region(region, &Resize::Top(top))?;
                self.resize_region(region, &Resize::Left(left))?;
            }
            Resize::TopRight(top, right) => {
                self.resize_region(region, &Resize::Top(top))?;
                self.resize_region(region, &Resize::Right(right))?;
            }
            Resize::BottomLeft(bottom, left) => {
                self.resize_region(region, &Resize::Bottom(bottom))?;
                self.resize_region(region, &Resize::Left(left))?;
            }
            Resize::BottomRight(bottom, right) => {
                self.resize_region(region, &Resize::Bottom(bottom))?;
                self.resize_region(region, &Resize::Right(right))?;
            }
        }

        Ok(())
    }

    pub fn swap_region(&mut self, region: &mut Region, direction: &Direction) -> Result<()> {
        let index = self
            .major_adjacent_region(region, direction)
            .ok_or(ErrorKind::NoAdjacentRegions)?;
        let sibling = self.regions.get_mut(index).unwrap();

        std::mem::swap(&mut region.size, &mut sibling.size);
        std::mem::swap(&mut region.pos, &mut sibling.pos);

        Ok(())
    }

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
