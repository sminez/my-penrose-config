use crate::{MAX_MAIN, RATIO, RATIO_STEP};
use penrose::{
    builtin::layout::{CenteredMain, Grid, MainAndStack, Monocle},
    core::layout::{Layout, LayoutStack},
    extensions::layout::{Conditional, Tatami},
    stack,
};

pub fn layouts() -> LayoutStack {
    stack!(
        flex_tall(),
        flex_wide(),
        MainAndStack::side(MAX_MAIN, RATIO, RATIO_STEP),
        Tatami::boxed(RATIO, RATIO_STEP),
        Grid::boxed(),
        Monocle::boxed()
    )
}

fn flex_tall() -> Box<dyn Layout> {
    Conditional::boxed(
        "FlexTall",
        MainAndStack::side_unboxed(MAX_MAIN, RATIO, RATIO_STEP, false),
        CenteredMain::vertical_unboxed(MAX_MAIN, RATIO, RATIO_STEP),
        |_, r| r.w <= 1400,
    )
}

fn flex_wide() -> Box<dyn Layout> {
    Conditional::boxed(
        "FlexWide",
        MainAndStack::bottom_unboxed(MAX_MAIN, RATIO, RATIO_STEP, false),
        CenteredMain::horizontal_unboxed(MAX_MAIN, RATIO, RATIO_STEP),
        |_, r| r.w <= 1400,
    )
}
