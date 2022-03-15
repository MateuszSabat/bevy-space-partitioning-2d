use std::ops::{Index, IndexMut};
use bevy::math::{Vec2};

// === Coords

pub struct Coords {
    pub x: i32,
    pub y: i32
}

impl Coords {
    pub fn new(x: i32, y: i32) -> Coords {
        Coords { x, y }
    }

    pub fn zero() -> Coords {
        Coords { x: 0, y: 0 }
    }

    pub fn translate(&self, offset: &Coords) -> Coords {
        Coords::new(self.x + offset.x, self.y + offset.y)
    }

    pub fn to_left(&self) -> Coords { Coords::new(self.x-1, self.y) }
    pub fn to_right(&self) -> Coords { Coords::new(self.x+1, self.y) }
    pub fn to_top_left(&self) -> Coords { Coords::new(self.x-1, self.y+1)}
    pub fn to_top_right(&self) -> Coords { Coords::new(self.x, self.y+1) }
    pub fn to_bottom_left(&self) -> Coords { Coords::new(self.x, self.y-1) }
    pub fn to_bottom_right(&self) -> Coords { Coords::new(self.x+1, self.y-1) }


    pub fn around(x: i32, y: i32, size: i32) -> impl Iterator<Item = Coords> {
        NeighbourCoords::new(x, y, size)
    }
    pub fn around_coords(coords: &Coords, size: i32) -> impl Iterator<Item = Coords> {
        Coords::around(coords.x, coords.y, size)
    }
}

impl PartialEq for Coords {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

// === Neighbourhood

struct NeighbourCoords {
    center: Coords,
    size: i32,
    offset: Coords,
}

impl NeighbourCoords {
    fn new(x: i32, y: i32, size: i32) -> NeighbourCoords {
        NeighbourCoords { center: Coords::new(x, y), size, offset: Coords::new(-1, -size) }
    }
}

impl Iterator for NeighbourCoords {
    type Item = Coords;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset.y < 0 {
            if self.offset.x < self.size {
                self.offset.x += 1
            } else {
                self.offset.y += 1;
                self.offset.x = -(self.size + self.offset.y)
            }
        } else {
            if self.offset.x < self.size - self.offset.y {
                self.offset.x += 1
            } else {
                self.offset.y += 1;
                self.offset.x = -self.size
            }
        }

        if self.offset.y > self.size {
            None
        } else {
            Some(self.center.translate(&self.offset))
        }
    }
}

// === Grid

pub struct Grid<D: Clone> {
    chunks: Axis<Axis<D>>
}

impl<D: Clone> Grid<D> {
    pub fn new(default: D) -> Grid<D> {
        Grid { chunks: Axis::<Axis<D>>::new(Axis::<D>::new(default)), }
    }

    pub fn ensure_bounds(&mut self, x_min: i32, x_max: i32, y_min: i32, y_max: i32) {
        let min = &mut self[&Coords::new(x_min, y_min)];
        let max = &mut self[&Coords::new(x_max, y_max)];
    }

    pub fn with_ensured_bounds(mut self, x_min: i32, x_max: i32, y_min: i32, y_max: i32) -> Grid<D> {
        self.ensure_bounds(x_min, x_max, y_min, y_max);
        self
    }
}

impl<D: Clone> Index<&Coords> for Grid<D> {
    type Output = D;

    fn index(&self, index: &Coords) -> &Self::Output {
        &self.chunks[index.x][index.y]
    }
}

impl<D: Clone> IndexMut<&Coords> for Grid<D> {
    fn index_mut(&mut self, index: &Coords) -> &mut Self::Output {
        &mut self.chunks[index.x][index.y]
    }
}

// === Transformations

fn grid_to_world_x() -> Vec2 { Vec2::new(1.0, 0.0) }
fn grid_to_world_y() -> Vec2 { Vec2::new(0.5, 0.8660254) }

pub fn grid_to_world(x: i32, y: i32) -> Vec2 {
    grid_to_world_x() * (x as f32) + grid_to_world_y() * (y as f32)
}

pub fn coords_to_world(coords: &Coords) -> Vec2 {
    grid_to_world(coords.x, coords.y)
}

pub fn world_to_grid(x: f32, y: f32) -> (i32, i32) {
    let grid_to_world_y = grid_to_world_y();

    let grid_y: f32 = y / grid_to_world_y.y;
    let grid_x: f32 = x - grid_y * grid_to_world_y.x;

    let distance_fn = |coords: &Coords| coords_to_world(coords).distance_squared(Vec2::new(x, y));

    let mut hex_x = grid_x as i32;
    let mut hex_y = grid_y as i32;
    let mut distance = distance_fn(&Coords::new(hex_x, hex_y));

    for coords in NeighbourCoords::new(hex_x, hex_y, 1) {
        let new_distance = distance_fn(&coords);
        if new_distance < distance {
            distance = new_distance;
            hex_x = coords.x;
            hex_y = coords.y;
        }
    }

    (hex_x, hex_y)
}

pub fn world_to_coords(x: f32, y: f32) -> Coords {
    let grid = world_to_grid(x, y);
    Coords::new(grid.0, grid.1)
}

// === Axis

struct Axis<E: Clone> {
    positive: Vec<E>,
    negative: Vec<E>,
    default: E,
}

impl<E: Clone> Axis<E> {
    fn new(default: E) -> Axis<E> {
        Axis {
            positive: Vec::<E>::new(),
            negative: Vec::<E>::new(),
            default
        }
    }
}

impl<E: Clone> Clone for Axis<E> {
    fn clone(&self) -> Self {
        let mut positive = Vec::<E>::new();
        for e in self.positive.iter() {
            positive.push(e.clone())
        }

        let mut negative = Vec::<E>::new();
        for e in self.negative.iter() {
            negative.push(e.clone())
        }

        Axis { positive, negative, default: self.default.clone() }
    }
}

impl<E: Clone> Index<i32> for Axis<E> {
    type Output = E;

    fn index(&self, index: i32) -> &Self::Output {
        if index >= 0 {
            let i = index as usize;
            if i >= self.positive.len() { &self.default } else { &self.positive[i] }
        } else {
            let i = (-index - 1) as usize;
            if i >= self.negative.len() { &self.default } else { &self.negative[i] }
        }
    }
}

impl<E: Clone> IndexMut<i32> for Axis<E> {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        if index >= 0 {
            let i = index as usize;
            self.positive.resize(i + 10, self.default.clone());
            &mut self.positive[i]
        } else {
            let i = (-index - 1) as usize;
            self.negative.resize(i + 10, self.default.clone());
            &mut self.negative[i]
        }
    }
}