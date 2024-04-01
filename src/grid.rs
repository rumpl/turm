use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use crate::{ansi::Color, row::Row};

#[derive(Debug)]
pub struct Grid {
    rows: Vec<Row>,
    scrollback: Vec<Row>,
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
            columns,
        }
    }

    /// Scrolls the grid up by one
    pub fn scroll_up(&mut self) {
        let len = self.rows.len();
        for i in 1..len {
            self.rows.swap(i - 1, i);
        }
        self.scrollback.push(self.rows[len - 1].clone());
        self.rows[len - 1].reset();
    }

    /// Scrolls the grid down by one, taking the last row from the scrollback
    pub fn scroll_down(&mut self) {
        let len = self.rows.len();
        for i in (1..len).rev() {
            self.rows.swap(i - 1, i);
        }
        self.rows[0] = self.scrollback.pop().unwrap();
    }

    pub fn sections(&self) -> Vec<TextSection> {
        let mut res = vec![];

        let mut current_fg = self.rows[0][0].fg;
        let mut current_bg = self.rows[0][0].bg;
        let mut text = String::new();

        for row in &self.rows {
            for col in &row.inner {
                if col.fg != current_fg || col.bg != current_bg {
                    let ts = TextSection {
                        text: text.clone(),
                        fg: current_fg,
                        bg: current_bg,
                    };
                    current_fg = col.fg;
                    current_bg = col.bg;
                    text = "".to_string();

                    res.push(ts);
                }
                text.push_str(&String::from(col.c));
            }
            text.push('\n');
        }

        if !text.is_empty() {
            let ts = TextSection {
                text: text.clone(),
                fg: current_fg,
                bg: current_bg,
            };
            res.push(ts);
        }

        res
    }

    fn print_vec(&self, v: &[Row], f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "|")?;
        for _ in 0..self.columns {
            write!(f, "_")?;
        }
        writeln!(f, "|")?;

        for row in v {
            write!(f, "|")?;
            for cell in &row.inner {
                if cell.c == '\t' {
                    write!(f, " ")?;
                } else {
                    write!(f, "{}", cell.c)?;
                }
            }
            writeln!(f, "|")?;
        }

        write!(f, "|")?;
        for _ in 0..self.columns {
            write!(f, "_")?;
        }
        writeln!(f, "|")?;

        Ok(())
    }
}

pub struct TextSection {
    pub text: String,
    pub fg: Color,
    pub bg: Color,
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
        writeln!(f, "\n\n#################################3\n\n")?;
        self.print_vec(&self.scrollback, f)?;
        writeln!(f, "-------------------------------------")?;
        self.print_vec(&self.rows, f)?;

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
}
