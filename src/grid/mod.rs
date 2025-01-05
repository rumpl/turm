use std::{
    cmp::min,
    fmt::Display,
    ops::{Index, IndexMut},
};

use self::cell::{Cell, Style};
use self::row::Row;

pub mod cell;
pub mod row;

#[derive(Debug)]
pub struct Grid {
    rows: Vec<Row>,
    scrollback: Vec<Row>,
    scrolldown: Vec<Row>,
    index: usize,
    columns: usize,
}

impl Grid {
    pub fn new(columns: usize, lines: usize) -> Self {
        let mut rows = Vec::with_capacity(lines);
        rows.resize(lines, Row::new(columns));

        Self {
            rows,
            index: 0,
            scrollback: vec![],
            scrolldown: vec![],
            columns,
        }
    }

    /// Scrolls the grid up by one
    pub fn scroll_up(&mut self, delta: u32, force: bool) {
        let d = min(delta, 4);
        for _ in 0..d {
            if !force && self.scrolldown.is_empty() {
                return;
            }
            if self.scrolldown.is_empty() {
                self.scrolldown.push(Row::new(self.columns));
            }
            let len = self.rows.len();
            for i in 1..len {
                self.rows.swap(i - 1, i);
            }
            self.scrollback.push(self.rows[len - 1].clone());
            self.rows[len - 1] = self.scrolldown.pop().unwrap();
        }
    }

    /// Scrolls the grid down by one, taking the last row from the scrollback
    pub fn scroll_down(&mut self, delta: u32, force: bool) {
        let d = min(delta, 4);
        for _ in 0..d {
            if !force && self.scrollback.is_empty() {
                return;
            }
            let len = self.rows.len();
            self.scrolldown.push(self.rows[len - 1].clone());
            for i in (1..len).rev() {
                self.rows.swap(i - 1, i);
            }
            self.rows[0] = self.scrollback.pop().unwrap();
        }
    }

    /// Returns the different style sections to render.
    pub fn sections(&self) -> Sections {
        let mut res = vec![];

        let mut current_style = self.rows[0][0].style;
        let mut whole_text =
            Vec::with_capacity(4 * (self.rows.len() * self.columns + self.rows.len()));

        let mut offset = 0;
        let mut len = 0;
        let mut total_len = 0;
        for row in &self.rows {
            for col in &row.inner {
                if col.style != current_style {
                    res.push(TextSection {
                        style: current_style,
                        offset,
                        len: offset + len,
                    });
                    offset += len;
                    len = 0;
                    current_style = col.style;
                }
                if let Some(c) = col.c {
                    whole_text.push(c);
                    len += c.len_utf8();
                    total_len += c.len_utf8();
                } else {
                    whole_text.push(' ');
                    len += 1;
                    total_len += 1;
                }
            }
            whole_text.push('\n');
            len += 1;
            total_len += 1;
        }

        if len != whole_text.len() {
            res.push(TextSection {
                style: current_style,
                offset,
                len: total_len,
            });
        }

        if res.is_empty() {
            res.push(TextSection {
                style: current_style,
                offset: 0,
                len: total_len,
            });
        }

        Sections {
            text: whole_text.iter().collect(),
            sections: res,
        }
    }

    pub fn resize(&mut self, new_columns: usize, new_lines: usize) {
        let mut new_rows: Vec<Row> = Vec::new();
        let mut current_row = Row::new(new_columns);
        let mut current_column_index = 0;

        // Flatten all cells from existing rows into a single vector
        let all_cells: Vec<Cell> = self.rows.iter().flat_map(|r| r.inner.clone()).collect();

        let mut advance = false;
        // Wrap cells into new rows based on the new column width
        for cell in all_cells {
            if advance && cell.c.is_none() {
                continue;
            } else {
                advance = false;
            }

            // If we arrived at the end of the row
            if current_column_index == new_columns {
                new_rows.push(current_row);
                current_row = Row::new(new_columns);
                current_column_index = 0;
            }

            if cell.c.is_some() {
                current_row[current_column_index] = cell;
                current_column_index += 1;
            } else {
                new_rows.push(current_row);
                current_row = Row::new(new_columns);
                current_column_index = 0;
                advance = true;
            }

            // Break out of the loop early if we've filled up the new_lines
            if new_rows.len() == new_lines {
                break;
            }
        }

        // Add the last row if it's not empty and we haven't exceeded new_lines
        if !current_row.inner.is_empty() && new_rows.len() < new_lines {
            new_rows.push(current_row);
        }

        // If the new size has more lines than the current content, add empty rows
        while new_rows.len() < new_lines {
            new_rows.push(Row::new(new_columns));
        }

        // Update grid rows and columns
        self.rows = new_rows;
        self.columns = new_columns; // Update the column count
    }

    fn print_vec(&self, v: &[Row], f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for _ in 0..self.columns {
            write!(f, "_")?;
        }
        writeln!(f)?;

        for row in v {
            write!(f, "|")?;
            for cell in &row.inner {
                if let Some(c) = cell.c {
                    if c == '\t' {
                        write!(f, " ")?;
                    } else {
                        write!(f, "{}", c)?;
                    }
                }
            }
            writeln!(f, "|")?;
        }

        for _ in 0..self.columns {
            write!(f, "-")?;
        }

        writeln!(f)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Sections {
    pub text: String,
    pub sections: Vec<TextSection>,
}

#[derive(Debug)]
pub struct TextSection {
    pub style: Style,
    pub offset: usize,
    pub len: usize,
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
        writeln!(f, "\n\n#################################\n\n")?;
        writeln!(f, "Scrollback")?;
        self.print_vec(&self.scrollback, f)?;
        writeln!(f, "-------------------------------------")?;
        writeln!(f, "Rows")?;
        self.print_vec(&self.rows, f)?;
        writeln!(f, "-------------------------------------")?;
        writeln!(f, "Scrolldown")?;
        self.print_vec(&self.scrolldown, f)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scroll_up() {
        let mut g = Grid::new(2, 2);
        g[1][0].c = Some('a');
        assert!(g[0][0].c.is_none());
        g.scroll_up(1, true);
        assert!(g[0][0].c == Some('a'));
    }

    #[test]
    fn test_resize() {
        let mut g = Grid::new(2, 2);
        g[0][0].c = Some('a');
        g[1][0].c = Some('b');

        g.resize(3, 2);

        assert!(g[0][0].c == Some('a'));
        assert!(g[1][0].c == Some('b'));
    }

    #[test]
    fn test_resize_with_empty() {
        let mut g = Grid::new(2, 2);
        g[0][0].c = Some('a');
        g[1][0].c = Some(' ');
        g[1][1].c = Some('a');

        println!("{}", g);
        g.resize(3, 3);
        println!("{}", g);

        assert!(g[1][1].c == Some('a'));
    }
}
