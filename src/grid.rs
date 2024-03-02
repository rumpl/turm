use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use crate::{gui::Cursor, row::Row};

#[derive(Debug)]
pub struct Grid {
    pub cursor: Cursor,
    columns: usize,
    lines: usize,
    rows: Vec<Row>,
    index: usize,
}

impl Grid {
    pub fn new(columns: usize, lines: usize) -> Self {
        let mut rows = Vec::with_capacity(lines);
        rows.resize(lines, Row::new(columns));

        Self {
            cursor: Cursor::default(),
            columns,
            lines,
            rows,
            index: 0,
        }
    }

    /// Scrolls the grid up by one
    pub fn scroll_up(&mut self) {
        for i in 1..self.rows.len() {
            self.rows.swap(i - 1, i);
        }
        let len = self.rows.len();
        self.rows[len - 1].reset();
    }

    pub fn scroll_down(&mut self) {
        for i in 1..self.rows.len() {
            self.rows.swap(i, i - 1);
        }
        self.rows[0].reset();
    }

    pub fn resize(&mut self, rows: usize, columns: usize) {
        println!("Resizing to {rows}, {columns}");
    }
}

impl Index<usize> for Grid {
    type Output = Row;

    fn index(&self, index: usize) -> &Self::Output {
        &self.rows[index]
    }
}

impl IndexMut<usize> for Grid {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.rows[index]
    }
}

impl Iterator for Grid {
    type Item = Row;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.rows.len() {
            let row = &self.rows[self.index];
            self.index += 1;
            Some(row.clone())
        } else {
            None
        }
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.rows {
            write!(f, "|")?;
            for cell in &row.inner {
                write!(f, "{}", cell.c)?;
            }
            write!(f, "|\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scroll_up() {
        let mut g = Grid::new(2, 2);
        g[1][0].c = 'a';
        assert!(g[0][0].c == ' ');
        g.scroll_up();
        assert!(g[0][0].c == 'a');
    }

    #[test]
    fn test_scoll_down() {
        let mut g = Grid::new(2, 2);
        g[0][0].c = 'a';
        assert!(g[1][0].c == ' ');
        g.scroll_down();
        assert!(g[1][0].c == 'a');
    }
}