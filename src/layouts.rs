use crate::{BAR_HEIGHT_PX, INNER_PX, MAX_MAIN, OUTER_PX, RATIO, RATIO_STEP};
use penrose::{
    builtin::layout::{
        transformers::{Gaps, ReserveTop},
        MainAndStack, Monocle,
    },
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
    .map(|layout| ReserveTop::wrap(Gaps::wrap(layout, OUTER_PX, INNER_PX), BAR_HEIGHT_PX))
}
