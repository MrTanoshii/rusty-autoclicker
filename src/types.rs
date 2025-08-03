use std::fmt;

use rdev::{Button, Key};

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum AppMode {
    Bot,
    Humanlike,
}

#[derive(PartialEq, Copy, Clone)]
pub struct ClickInfo {
    pub click_btn: ClickButton,
    pub click_coord: (f64, f64),
    pub click_position: ClickPosition,
    pub click_type: ClickType,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ClickPosition {
    Mouse,
    Coord,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ClickType {
    Single,
    Double,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ClickButton {
    Mouse(Button),
    Key(Key),
}

impl fmt::Display for ClickButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClickButton::Mouse(button) => write!(f, "{button:?}"),
            ClickButton::Key(key) => write!(f, "{key:?}"),
        }
    }
}
