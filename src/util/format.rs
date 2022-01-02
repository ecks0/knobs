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
    // Format as watts to one decimal place by default. So far,
    // I haven't seen a rapl-supporting CPU or nvidia card that
    // can run at less than 100 milliwatt/sec.
    if 0. == p.as_milliwatts().trunc() {
        "0 W".to_string()
    } else {
        let p = p.as_watts();
        let scale = 10.;
        let p = (p * scale).ceil()/scale;
        let p = Power::from_watts(p);
        p.to_string()
    }
}

#[derive(Debug)]
pub(crate) struct Table(ct::Table);

impl Table {
    pub(crate) fn new(header: &[&str]) -> Self {
        let mut tab = ct::Table::new();
        tab.load_preset(ct::presets::NOTHING);
        tab.set_header(header);
        tab.add_row(header.iter().map(|h| "-".repeat(h.len())).collect::<Vec<String>>());
        Self(tab)
    }

    pub(crate) fn row<S: Display>(&mut self, row: &[S]) {
        self.0.add_row(row);
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0)
    }
}
