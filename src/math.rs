use std::{fmt::Debug, marker::PhantomData};

use num::Num;

pub trait Point<U: Num, const Dim: usize> {
    fn to_array(&self) -> [U; Dim];
}

impl<U: Num + Clone + Debug, A: AsRef<[U; Dim]>, const Dim: usize> Point<U, Dim> for A {
    fn to_array(&self) -> [U; Dim] {
        self.as_ref().to_vec().try_into().unwrap()
    }
}

pub struct AABB<U: Num, P: Point<U, 3>> {
    start: P,
    end: P,
    _unit: PhantomData<U>,
}

impl<U: Num, P: Point<U, 3>> AABB<U, P> {
    fn new(start: P, end: P) -> Self {
        AABB {
            start,
            end,
            _unit: PhantomData::default(),
        }
    }

    fn dimensions(&self) -> [U; 3] {
        let [start_x, start_y, start_z] = self.start.to_array();
        let [end_x, end_y, end_z] = self.end.to_array();
        let x = end_x - start_x;
        let y = end_y - start_y;
        let z = end_z - start_z;
        [x, y, z]
    }
}
