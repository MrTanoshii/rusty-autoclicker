use rdev::Button;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum AppMode {
    Bot,
    Humanlike,
}

#[derive(PartialEq, Copy, Clone)]
pub struct ClickInfo {
    pub click_btn: Button,
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
