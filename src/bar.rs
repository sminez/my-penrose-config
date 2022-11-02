use crate::{BAR_HEIGHT_PX, BLACK, BLUE, FONT, GREY, MAX_ACTIVE_WINDOW_CHARS, WHITE};
use penrose::{x::XConn, Color};
use penrose_ui::{
    bar::{
        widgets::{ActiveWindowName, CurrentLayout, RootWindowName, Workspaces},
        Position, StatusBar,
    },
    core::TextStyle,
};

// Mostly the example dwm bar from the main repo but recreated here so it's easier to tinker
// with and add in debug widgets when needed.
pub fn status_bar<X: XConn>() -> penrose_ui::Result<StatusBar<X>> {
    let highlight: Color = BLUE.into();
    let empty_ws: Color = GREY.into();

    let style = TextStyle {
        font: FONT.to_string(),
        point_size: 8,
        fg: WHITE.into(),
        bg: Some(BLACK.into()),
        padding: (2.0, 2.0),
    };

    StatusBar::try_new(
        Position::Top,
        BAR_HEIGHT_PX,
        style.bg.unwrap_or_else(|| 0x000000.into()),
        &[&style.font],
        vec![
            Box::new(Workspaces::new(&style, highlight, empty_ws)),
            Box::new(CurrentLayout::new(&style)),
            // Box::new(penrose_bar::widgets::debug::StateSummary::new(style)),
            Box::new(ActiveWindowName::new(
                MAX_ACTIVE_WINDOW_CHARS,
                &TextStyle {
                    bg: Some(highlight),
                    padding: (6.0, 4.0),
                    ..style.clone()
                },
                true,
                false,
            )),
            Box::new(RootWindowName::new(
                &TextStyle {
                    padding: (4.0, 2.0),
                    ..style.clone()
                },
                false,
                true,
            )),
        ],
    )
}
