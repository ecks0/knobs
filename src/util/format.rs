use std::fmt::Display;

use comfy_table as ct;
use measurements::{Frequency, Power};

pub(crate) const DOT: &str = "\u{2022}";

pub(crate) fn dot() -> String {
    DOT.to_string()
}

pub(crate) fn frequency(f: Frequency) -> String {
    let h = f.as_hertz().trunc() as u64;
    if h == 0 {
        "0 Hz".to_string()
    } else if h < 10u64.pow(9) {
        format!("{:.0}", f)
    } else {
        format!("{:.1}", f)
    }
}

pub(crate) fn power(p: Power) -> String {
    let mw = p.as_milliwatts().trunc() as u64;
    if 0 == mw {
        "0 W".to_string()
    } else if mw < 1000 || mw % 1000 == 0 {
        format!("{:.0}", p)
    } else {
        format!("{:.1}", p)
    }
}

#[derive(Debug)]
pub(crate) struct Table<'a> {
    header: &'a [&'static str],
    rows: Vec<Vec<String>>,
}

impl<'a> Table<'a> {
    const LINE: &'static str = "-";

    pub(crate) fn new(header: &'a [&'static str]) -> Self {
        let rows = vec![];
        Self { header, rows }
    }

    pub(crate) fn row(&mut self, row: impl IntoIterator<Item = String>) {
        let row = row.into_iter().collect();
        self.rows.push(row);
    }

    pub(crate) fn rows<I, R>(&mut self, rows: I)
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = String>,
    {
        for row in rows {
            self.row(row);
        }
    }

    pub(crate) fn format(self) -> String {
        self.to_string()
    }
}

impl<'a> Display for Table<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let seps: Vec<_> = self
            .header
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let h = h.chars().count();
                let c = self.rows.iter().fold(h, |a, v| v[i].chars().count().max(a));
                Self::LINE.repeat(c)
            })
            .collect();
        let mut tab = ct::Table::new();
        tab.load_preset(ct::presets::NOTHING);
        tab.set_header(self.header);
        tab.add_row(seps);
        for row in &self.rows {
            tab.add_row(row);
        }
        writeln!(f, "{}", tab)
    }
}

impl<'a> From<Table<'a>> for String {
    fn from(v: Table<'a>) -> Self {
        v.to_string()
    }
}
