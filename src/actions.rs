use penrose::{
    core::{
        helpers::{spawn, spawn_for_output},
        hooks::Hook,
        manager::WindowManager,
        xconnection::XConn,
    },
    PenroseError, Result,
};

use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum XrandrMonitorPosition {
    /// Left of the primary monitor
    Left,
    /// Right of the primary monitor
    Right,
    /// Above the primary monitor
    Above,
    /// Below the primary monitor
    Below,
}

impl fmt::Display for XrandrMonitorPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            XrandrMonitorPosition::Left => "--left-of",
            XrandrMonitorPosition::Right => "--right-of",
            XrandrMonitorPosition::Above => "--above",
            XrandrMonitorPosition::Below => "--below",
        };
        write!(f, "{}", s)
    }
}

/*
 * Detect the current monitor set up and arrange the monitors if needed using xrandr.
 *
 * NOTE
 * - Primary monitor will be set to `primary`
 * - Monitor resolution is set using the --auto flag in xrandr
 * - Only supports one and two monitor setups.
 */
pub fn update_monitors_via_xrandr(
    primary: &str,
    secondary: &str,
    position: XrandrMonitorPosition,
) -> Result<()> {
    let raw = spawn_for_output("xrandr")?;
    let secondary_line =
        raw.lines()
            .find(|line| line.starts_with(secondary))
            .ok_or(PenroseError::Raw(
                "unable to find secondary monitor in xrandr output".into(),
            ))?;
    let status = secondary_line
        .split(' ')
        .nth(1)
        .ok_or(PenroseError::Raw("unexpected xrandr output".into()))?;

    // force the primary monitor
    spawn(format!("xrandr --output {} --primary --auto", primary));
    match status {
        "disconnected" => spawn(format!("xrandr --output {} --off", secondary)),
        "connected" => spawn(format!(
            "xrandr --output {} --auto {} {}",
            secondary, position, primary
        )),
        _ => (),
    }

    Ok(())
}

// Automatically set the current monitors and their positions whenever there is an xrandr change
#[derive(Clone, Debug)]
pub struct AutoSetMonitorsViaXrandr {
    primary: String,
    secondary: String,
    position: XrandrMonitorPosition,
}

impl AutoSetMonitorsViaXrandr {
    pub fn new<S>(primary: S, secondary: S, position: XrandrMonitorPosition) -> Box<Self>
    where
        S: Into<String>,
    {
        Box::new(Self {
            primary: primary.into(),
            secondary: secondary.into(),
            position,
        })
    }
}

impl<X> Hook<X> for AutoSetMonitorsViaXrandr
where
    X: XConn,
{
    fn randr_notify(&mut self, _: &mut WindowManager<X>) {
        if let Err(e) = update_monitors_via_xrandr(&self.primary, &self.secondary, self.position) {
            error!("{}", e);
        }
    }
}
