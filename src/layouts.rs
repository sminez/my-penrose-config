use crate::{
    bar::{BAR_HEIGHT_PX_EXTERNAL, BAR_HEIGHT_PX_PRIMARY},
    MAX_MAIN, RATIO, RATIO_STEP,
};
use penrose::{
    builtin::layout::{CenteredMain, Grid, MainAndStack, Monocle},
    core::{
        hooks::LayoutHook,
        layout::{Layout, LayoutStack},
        State,
    },
    extensions::layout::{Conditional, Tatami},
    pure::geometry::Rect,
    stack,
    x::XConn,
    Result, Xid,
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

/// A manage hook for positioning a client as pseudo-fullscreen while still retaining the status
/// bar.
pub fn full_screen_minus_bar<X: XConn>(client: Xid, state: &mut State<X>, _: &X) -> Result<()> {
    let screen = &state.client_set.current_screen();
    let mut r = screen.geometry();
    let dy = if screen.index() == 0 {
        BAR_HEIGHT_PX_PRIMARY
    } else {
        BAR_HEIGHT_PX_EXTERNAL
    };

    r.y += dy;
    r.h -= dy;

    state.client_set.float(client, r)
}

/// Modified version of the main crate ScreenSpacingHook that changes the spacing based on the
/// screen index so that I can give more space on my internal screen which has a higher resolution.
#[derive(Debug, Clone, Default)]
pub struct PerScreenSpacingHook {
    pub outer_px: u32,
    pub inner_px: u32,
}

impl<X: XConn> LayoutHook<X> for PerScreenSpacingHook {
    fn transform_initial_for_screen(
        &mut self,
        screen_index: usize,
        mut r: Rect,
        _: &State<X>,
        _: &X,
    ) -> Rect {
        if r.w == 0 || r.h == 0 {
            return r;
        }

        let top_px = if screen_index == 0 {
            BAR_HEIGHT_PX_PRIMARY
        } else {
            BAR_HEIGHT_PX_EXTERNAL
        };

        r.y += top_px;
        r.h -= top_px;

        shrink(r, self.outer_px)
    }

    fn transform_positions(
        &mut self,
        _: Rect,
        positions: Vec<(Xid, Rect)>,
        _: &State<X>,
        _: &X,
    ) -> Vec<(Xid, Rect)> {
        positions
            .into_iter()
            .map(|(id, r)| (id, shrink(r, self.inner_px)))
            .collect()
    }
}

fn shrink(r: Rect, px: u32) -> Rect {
    if r.w == 0 || r.h == 0 {
        return r;
    }

    Rect {
        x: r.x + px,
        y: r.y + px,
        w: r.w - 2 * px,
        h: r.h - 2 * px,
    }
}
