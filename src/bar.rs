use crate::{BAR_HEIGHT_PX, BLACK, BLUE, FONT, GREY, MAX_ACTIVE_WINDOW_CHARS, WHITE};
use penrose::{x::XConn, Color};
use penrose_ui::{
    bar::{
        widgets::{ActiveWindowName, CurrentLayout, RootWindowName, Spacer, Workspaces},
        Position, StatusBar,
    },
    TextStyle,
};

// Mostly the example dwm bar from the main repo but recreated here so it's easier to tinker
// with and add in debug widgets when needed.
pub fn status_bar<X: XConn>() -> penrose_ui::Result<StatusBar<X>> {
    let highlight: Color = BLUE.into();
    let empty_ws: Color = GREY.into();

    let style = TextStyle {
        fg: WHITE.into(),
        bg: Some(BLACK.into()),
        padding: (2, 2),
    };

    StatusBar::try_new(
        Position::Top,
        BAR_HEIGHT_PX,
        style.bg.unwrap_or_else(|| 0x000000.into()),
        FONT,
        12,
        vec![
            Box::new(Workspaces::new(style, highlight, empty_ws)),
            Box::new(CurrentLayout::new(style)),
            Box::new(ActiveWindowName::new(
                MAX_ACTIVE_WINDOW_CHARS,
                TextStyle {
                    bg: Some(highlight),
                    padding: (6, 4),
                    ..style
                },
                true,
                false,
            )),
            Box::new(RootWindowName::new(style, false, true)),
            Box::new(Spacer::new(vec![0], 0.07)), // reserve space for trayer
        ],
    )
}
