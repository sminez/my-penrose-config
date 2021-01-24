use penrose::{
    core::{
        client::Client, data_types::WinId, hooks::Hook, manager::WindowManager, xconnection::XConn,
    },
    Result,
};

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

    fn centered_above<X: XConn>(&self, id: WinId, wm: &mut WindowManager<X>) -> Result<()> {
        if let Some(region) = wm.screen_size(wm.active_screen_index()) {
            let r = region.scale_w(self.scale).scale_h(self.scale);
            wm.position_client(id, r.centered_in(&region)?, true)?;
        }
        wm.show_client(id)
    }
}

impl<X: XConn> Hook<X> for CenterFloat {
    fn new_client(&mut self, wm: &mut WindowManager<X>, c: &mut Client) -> Result<()> {
        if c.wm_class() == self.class_name {
            c.set_floating(true);
            self.centered_above(c.id(), wm)?;
        }

        Ok(())
    }
}
