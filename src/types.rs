#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ClickType {
    Single,
    Double,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ClickPosition {
    Mouse,
    Coord,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum AppMode {
    Bot,
    Humanlike,
}
