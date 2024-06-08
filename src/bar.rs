use crate::{schedule::CronRunner, BLACK, BLUE, FONT, GREY, WHITE};
use penrose::{
    core::State,
    pure::geometry::{Point, Rect},
    util::{spawn_for_output, spawn_for_output_with_args},
    x::XConn,
    Color,
};
use penrose_ui::{
    bar::{
        widgets::{ActiveWindowName, CurrentLayout, Widget, Workspaces},
        PerScreen, Position, StatusBar,
    },
    Context, Result, TextStyle,
};
use std::{fs, time::Duration};

pub const MAX_ACTIVE_WINDOW_CHARS: usize = 50;
pub const BAR_HEIGHT_PX_PRIMARY: u32 = 24;
pub const BAR_HEIGHT_PX_EXTERNAL: u32 = 18;
pub const BAR_POINT_SIZE_PRIMARY: u8 = 12;
pub const BAR_POINT_SIZE_EXTERNAL: u8 = 8;

fn base_widgets<X: XConn>(runner: &mut CronRunner) -> Vec<Box<dyn Widget<X>>> {
    let highlight: Color = BLUE.into();
    let empty_ws: Color = GREY.into();
    let style = TextStyle {
        fg: WHITE.into(),
        bg: Some(BLACK.into()),
        padding: (2, 2),
    };

    let pstyle = TextStyle {
        padding: (5, 5),
        ..style
    };

    let ms = |n: u64| Duration::from_millis(n);

    vec![
        Box::new(Wedge::start(BLUE, BLACK)),
        Box::new(Workspaces::new(style, highlight, empty_ws)),
        Box::new(CurrentLayout::new(style)),
        Box::new(Wedge::end(BLUE, BLACK).only_with_focus()),
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
        Box::new(Wedge::start(BLUE, BLACK).only_with_focus()),
        // The wttr.in API is freaking out a bit recently and hanging / returning errors
        // so dropping this for now.
        // Box::new(runner.new_cron_text(pstyle, weather_text, ms(300_000))),
        Box::new(runner.new_cron_text(pstyle, wifi_text, ms(10_000))),
        Box::new(runner.new_cron_text(pstyle, || battery_text("BAT1"), ms(60_000))),
        Box::new(runner.new_cron_text(pstyle, || amixer_text("Master"), ms(1000))),
        Box::new(runner.new_cron_text(pstyle, date_text, ms(10_000))),
    ]
}

pub fn status_bar<X: XConn>() -> Result<StatusBar<X>> {
    let mut runner = CronRunner::default();
    let mut primary = base_widgets(&mut runner);
    primary.push(Box::new(Spacer::new(0.07))); // reserve space for trayer
    let external = base_widgets(&mut runner);

    let bar = StatusBar::try_new_per_screen(
        Position::Top,
        BLACK,
        FONT,
        vec![
            PerScreen::new(BAR_POINT_SIZE_PRIMARY, BAR_HEIGHT_PX_PRIMARY, primary),
            PerScreen::new(BAR_POINT_SIZE_EXTERNAL, BAR_HEIGHT_PX_EXTERNAL, external),
        ],
    )?;

    runner.run_threaded();

    Ok(bar)
}

/// A simple 45 degree wedge
#[derive(Debug, Clone, Copy)]
pub struct Wedge {
    only_with_focus: bool,
    start: bool,
    fg: Color,
    bg: Color,
}

impl Wedge {
    fn new(fg: impl Into<Color>, bg: impl Into<Color>, start: bool) -> Self {
        Self {
            only_with_focus: false,
            start,
            fg: fg.into(),
            bg: bg.into(),
        }
    }

    fn start(fg: impl Into<Color>, bg: impl Into<Color>) -> Self {
        Self::new(fg, bg, true)
    }

    fn end(fg: impl Into<Color>, bg: impl Into<Color>) -> Self {
        Self::new(fg, bg, false)
    }

    fn only_with_focus(mut self) -> Self {
        self.only_with_focus = true;
        self
    }
}

impl<X: XConn> Widget<X> for Wedge {
    fn draw(&mut self, ctx: &mut Context<'_>, _: usize, f: bool, w: u32, h: u32) -> Result<()> {
        ctx.fill_rect(Rect::new(0, 0, w, h), self.bg)?;
        if self.only_with_focus && !f {
            return Ok(());
        }

        let p = if self.start { 0 } else { h };
        ctx.fill_polygon(
            &[Point::new(p, p), Point::new(h, 0), Point::new(0, h)],
            self.fg,
        )
    }

    fn current_extent(&mut self, _: &mut Context<'_>, h: u32) -> Result<(u32, u32)> {
        Ok((h, h))
    }

    fn is_greedy(&self) -> bool {
        false
    }

    fn require_draw(&self) -> bool {
        false
    }
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

pub fn date_text() -> Option<String> {
    Some(
        spawn_for_output_with_args("date", &["+%F %R"])
            .unwrap_or_default()
            .trim()
            .to_string(),
    )
}

pub fn weather_text() -> Option<String> {
    Some(
        spawn_for_output_with_args(
            "curl",
            &["-s", "--max-time", "5", "http://wttr.in?format=%c%t"],
        )
        .unwrap_or_default()
        .trim()
        .to_string(),
    )
}

pub fn battery_text(bat: &str) -> Option<String> {
    let status = read_sys_file(bat, "status")?;
    let energy_now: u32 = read_sys_file(bat, "charge_now")?.parse().ok()?;
    let energy_full: u32 = read_sys_file(bat, "charge_full")?.parse().ok()?;

    let charge = energy_now * 100 / energy_full;

    let icon = if status == "Charging" {
        ""
    } else if charge >= 90 || status == "Full" {
        ""
    } else if charge >= 70 {
        ""
    } else if charge >= 50 {
        ""
    } else if charge >= 20 {
        ""
    } else {
        ""
    };

    Some(format!("{icon} {charge}%"))
}

fn read_sys_file(bat: &str, fname: &str) -> Option<String> {
    fs::read_to_string(format!("/sys/class/power_supply/{bat}/{fname}"))
        .ok()
        .map(|s| s.trim().to_string())
}

pub fn wifi_text() -> Option<String> {
    let (interface, essid) = interface_and_essid()?;
    let signal = signal_quality(&interface)?;

    Some(format!("<{essid} {signal}%>"))
}

// Read the interface name and essid via iwgetid.
//   Output format is '$interface    ESSID:"$essid"'
fn interface_and_essid() -> Option<(String, String)> {
    let raw = spawn_for_output("iwgetid").ok()?;
    let mut iter = raw.split(':');

    // Not using split_whitespace here as the essid may contain whitespace
    let interface = iter.next()?.split_whitespace().next()?.to_owned();
    let essid = iter.next()?.split('"').nth(1)?.to_string();

    Some((interface, essid))
}

// Parsing the format described here: https://hewlettpackard.github.io/wireless-tools/Linux.Wireless.Extensions.html
fn signal_quality(interface: &str) -> Option<String> {
    let raw = fs::read_to_string("/proc/net/wireless").ok()?;

    for line in raw.lines() {
        if line.starts_with(interface) {
            return Some(
                line.split_whitespace()
                    .nth(2)?
                    .strip_suffix('.')?
                    .to_owned(),
            );
        }
    }

    None
}

// Parse the current volume as a percentage from amixer.
//
// Expected output format:
//   $ amixer sget Master
//     Simple mixer control 'Master',0
//       Capabilities: pvolume pvolume-joined pswitch pswitch-joined
//       Playback channels: Mono
//       Limits: Playback 0 - 127
//       Mono: Playback 0 [0%] [-63.50dB] [on]
pub fn amixer_text(channel: &str) -> Option<String> {
    let raw = spawn_for_output(format!("amixer sget {channel}")).ok()?;

    let vol = raw
        .lines()
        .last()?
        .split_whitespace()
        .find(|s| s.ends_with("%]"))?
        .replace(|c| "[]%".contains(c), "");

    Some(format!(" {vol}%"))
}
