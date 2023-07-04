use crate::{MAX_MAIN, RATIO, RATIO_STEP};
use penrose::{
    builtin::layout::{CenteredMain, Grid, MainAndStack, Monocle},
    core::layout::LayoutStack,
    extensions::layout::Tatami,
    stack,
};

pub fn layouts() -> LayoutStack {
    stack!(
        MainAndStack::side(MAX_MAIN, RATIO, RATIO_STEP),
        MainAndStack::bottom(MAX_MAIN, RATIO, RATIO_STEP),
        CenteredMain::vertical(MAX_MAIN, RATIO, RATIO_STEP),
        Tatami::boxed(RATIO, RATIO_STEP),
        Grid::boxed(),
        Monocle::boxed()
    )
}
