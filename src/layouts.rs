use crate::{MAX_MAIN, RATIO, RATIO_STEP};
use penrose::{
    builtin::layout::{MainAndStack, Monocle},
    core::layout::LayoutStack,
    stack,
};

pub fn layouts() -> LayoutStack {
    stack!(
        MainAndStack::side(MAX_MAIN, RATIO, RATIO_STEP),
        MainAndStack::side_mirrored(MAX_MAIN, RATIO, RATIO_STEP),
        MainAndStack::bottom(MAX_MAIN, RATIO, RATIO_STEP),
        Monocle::boxed()
    )
}
