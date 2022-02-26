use penrose::{
    core::{
        data_types::Region,
        hooks::Hook,
        manager::WindowManager,
        xconnection::{XConn, Xid},
    },
    draw::{Color, DrawContext, Widget},
    Result, Selector,
};

pub struct StartupScript {
    path: String,
}
impl StartupScript {
    pub fn new(s: impl Into<String>) -> Self {
        Self { path: s.into() }
    }
}

impl<X: XConn> Hook<X> for StartupScript {
    fn startup(&mut self, _: &mut WindowManager<X>) -> Result<()> {
        spawn!(&self.path)
    }
}

pub struct CenterFloat {
    class_name: String,
    scale: f64,
}

impl CenterFloat {
    pub fn new(class_name: impl Into<String>, scale: f64) -> Box<Self> {
        Box::new(Self {
            class_name: class_name.into(),
            scale,
        })
    }

    fn centered_above<X: XConn>(&self, id: Xid, wm: &mut WindowManager<X>) -> Result<()> {
        if let Some(region) = wm.screen_size(wm.active_screen_index()) {
            let r = region.scale_w(self.scale).scale_h(self.scale);
            wm.position_client(id, r.centered_in(&region)?, true)?;
        }
        wm.show_client(id)
    }
}

impl<X: XConn> Hook<X> for CenterFloat {
    fn new_client(&mut self, wm: &mut WindowManager<X>, id: u32) -> Result<()> {
        if let Some(c) = wm.client_mut(&id.into()) {
            if c.wm_class() == self.class_name {
                c.set_floating(true);
                self.centered_above(c.id(), wm)?;
            }
        }

        Ok(())
    }
}

/// A wrapper that spawns and captures a stalonetray window
#[derive(Clone, Debug, PartialEq)]
pub struct StaloneTray {
    id: Option<Xid>,
    r: Region,
    cmd: String,
    size_changed: bool,
    need_reposition: bool,
}

impl StaloneTray {
    /// Create a new stalonetray wrapper that will ensure that the window is correctly positioned
    pub fn new(icon_size: usize, bg: impl Into<Color>) -> Result<Self> {
        Ok(Self {
            id: None,
            r: Region::default(),
            cmd: format!(
                "stalonetray -bg {} --icon-size {}",
                bg.into().as_rgb_hex_string(),
                icon_size
            ),
            size_changed: false,
            need_reposition: false,
        })
    }
}

impl<X> Hook<X> for StaloneTray
where
    X: XConn,
{
    fn startup(&mut self, _: &mut WindowManager<X>) -> Result<()> {
        spawn!(&self.cmd)
    }

    fn new_client(&mut self, wm: &mut WindowManager<X>, id: u32) -> Result<()> {
        if wm.client(&id.into()).map(|c| c.wm_name()) == Some("stalonetray") {
            self.id = Some(id);
        }

        Ok(())
    }

    // Check to see if our window size has changed and reposition if it has
    fn event_handled(&mut self, wm: &mut WindowManager<X>) -> Result<()> {
        if let Some(id) = self.id {
            let r = wm.conn().client_geometry(id)?;
            if r != self.r {
                let (x, _, w, _) = wm
                    .screen(&Selector::Index(0))
                    .expect("should always have at least one screen")
                    .region(false)
                    .values();
                let r = Region::new(x + w - r.x, 0, r.w, r.h);
                wm.conn().position_client(id, r, 0, true)?;
                self.size_changed = true;
                self.r = r;
            }
        }
        Ok(())
    }
}

type DResult<T> = penrose::draw::Result<T>;

impl Widget for StaloneTray {
    // just clear the flag
    fn draw(&mut self, _: &mut dyn DrawContext, _: usize, _: bool, _: f64, _: f64) -> DResult<()> {
        self.size_changed = false;
        Ok(())
    }

    // The current window size of stalonetray
    fn current_extent(&mut self, _: &mut dyn DrawContext, _: f64) -> DResult<(f64, f64)> {
        let (_, _, w, h) = self.r.values();
        Ok((w as f64, h as f64))
    }

    // If we previously repositioned then we need to report that back to the status bar
    fn require_draw(&self) -> bool {
        self.size_changed
    }

    // Only use space for stalonetray
    fn is_greedy(&self) -> bool {
        false
    }
}
