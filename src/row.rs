use std::ops::{Index, IndexMut};

use crate::cell::Cell;

#[derive(Debug, Clone)]
pub struct Row {
    pub inner: Vec<Cell>,
    index: usize,
}

impl Row {
    pub fn new(columns: usize) -> Self {
        let mut inner = Vec::with_capacity(columns);

        inner.resize(columns, Cell::new());

        Self { inner, index: 0 }
    }

    pub fn reset(&mut self) {
        for cell in &mut self.inner {
            cell.c = ' ';
        }
    }
}

impl Index<usize> for Row {
    type Output = Cell;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl IndexMut<usize> for Row {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl Iterator for Row {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.inner.len() {
            let cell = self.inner[self.index];
            self.index += 1;
            Some(cell)
        } else {
            None
        }
    }
}
