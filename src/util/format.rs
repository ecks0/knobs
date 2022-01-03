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
        // Format units less than gigahertz without decimals.
        format!("{:.0}", f)
    } else {
        // Format gigahertz and above to one decimal place.
        format!("{:.1}", f)
    }
}

pub(crate) fn power(p: Power) -> String {
    // Format power as watts to one decimal place by default.
    // So far, I haven't seen a rapl-capable CPU or a nvidia
    // card that can run at less than 100 milliwatt/sec.
    if 0. == p.as_milliwatts().trunc() {
        "0 W".to_string()
    } else {
        let p = p.as_watts();
        let scale = 10.;
        let p = (p * scale).ceil() / scale;
        let p = Power::from_watts(p);
        p.to_string()
    }
}

#[derive(Debug)]
pub struct Table<'a> {
    header: &'a [&'a str],
    rows: Vec<Vec<String>>,
}

impl<'a> Table<'a> {
    const LINE: &'static str = "-";

    pub(crate) fn new(header: &'a [&'a str]) -> Self {
        let rows = vec![];
        Self { header, rows }
    }

    pub(crate) fn row<S: Display>(&mut self, row: &[S]) {
        let row = row.iter().map(ToString::to_string).collect();
        self.rows.push(row);
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
        writeln!(f, "{}", tab.to_string())
    }
}
