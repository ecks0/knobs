use crate::app::{Arg, Parser};

const UNINSTALL: &str = "uninstall";
const DIR: &str = "dir";

const UNINSTALL_SHORT: char = 'u';

const UNINSTALL_HELP: &str = "Uninstall utility symlinks";
const DIR_HELP: &str = "Specify utility symlink directory";

pub(super) fn args() -> Vec<Arg> {
    vec![
        Arg {
            name: UNINSTALL.into(),
            long: UNINSTALL.into(),
            short: UNINSTALL_SHORT.into(),
            help: UNINSTALL_HELP.into(),
            ..Default::default()
        },
        Arg {
            name: DIR.into(),
            help: DIR_HELP.into(),
            ..Default::default()
        },
    ]
}

impl super::Values {
    pub(super) fn from_parser(p: Parser<'_>) -> Self {
        Self {
            uninstall: p.flag(UNINSTALL),
            dir: p.string(DIR),
        }
    }
}
