use crate::{BLACK, BLUE, FONT, GREY, WHITE};
use penrose::{core::State, pure::geometry::Rect, x::XConn, Color};
use penrose_ui::{
    bar::{
        widgets::{ActiveWindowName, CurrentLayout, RootWindowName, Widget, Workspaces},
        PerScreen, Position, StatusBar,
    },
    Context, Result, TextStyle,
};

pub const MAX_ACTIVE_WINDOW_CHARS: usize = 50;
pub const BAR_HEIGHT_PX_PRIMARY: u32 = 24;
pub const BAR_HEIGHT_PX_EXTERNAL: u32 = 18;
pub const BAR_POINT_SIZE_PRIMARY: u8 = 12;
pub const BAR_POINT_SIZE_EXTERNAL: u8 = 8;

fn base_widgets<X: XConn>() -> Vec<Box<dyn Widget<X>>> {
    let highlight: Color = BLUE.into();
    let empty_ws: Color = GREY.into();
    let style = TextStyle {
        fg: WHITE.into(),
        bg: Some(BLACK.into()),
        padding: (2, 2),
    };

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
    ]
}

pub fn status_bar<X: XConn>() -> Result<StatusBar<X>> {
    let mut primary = base_widgets();
    primary.push(Box::new(Spacer::new(0.07))); // reserve space for trayer

    StatusBar::try_new_per_screen(
        Position::Top,
        BLACK,
        FONT,
        vec![
            PerScreen::new(BAR_POINT_SIZE_PRIMARY, BAR_HEIGHT_PX_PRIMARY, primary),
            PerScreen::new(
                BAR_POINT_SIZE_EXTERNAL,
                BAR_HEIGHT_PX_EXTERNAL,
                base_widgets(),
            ),
        ],
    )
}

#[derive(Debug)]
pub struct Spacer {
    perc: f32,
    w: u32,
}

impl Spacer {
    pub fn new(perc: f32) -> Self {
        if !(0.0..=1.0).contains(&perc) {
            panic!("{perc} is an invalid percentage");
        }

        Self { perc, w: 0 }
    }
}

impl<X: XConn> Widget<X> for Spacer {
    fn draw(&mut self, ctx: &mut Context<'_>, _: usize, _: bool, w: u32, h: u32) -> Result<()> {
        ctx.fill_bg(Rect::new(0, 0, w, h))
    }

    fn current_extent(&mut self, _: &mut Context<'_>, h: u32) -> Result<(u32, u32)> {
        Ok((self.w, h))
    }

    fn is_greedy(&self) -> bool {
        false
    }

    fn require_draw(&self) -> bool {
        false
    }

    fn on_startup(&mut self, state: &mut State<X>, _: &X) -> Result<()> {
        self.w = state
            .client_set
            .screens()
            .next()
            .map(|s| (s.geometry().w as f32 * self.perc) as u32)
            .unwrap();

        Ok(())
    }
}
