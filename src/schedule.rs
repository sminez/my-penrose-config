//! This is a work in progress attempt at putting together a simple cron-like runner for handling
//! self updating status bar widgets that do not depend on window manager state.
//!
//! The [penrose_ui::bar::widgets::IntervalText] widget runs in a dedicated thread which is fine
//! for a single widget but pretty wasteful once there are multiple. The [CronRunner] struct in
//! this module instead acts as a single thread of execution for an arbitrary number of widgets
//! that are then updated specified interval. The updates are run on a best effort attempt to
//! honour the requested schedule but may end up drifting if individual widgets take too long to
//! run their update function.
use penrose::{util::spawn_with_args, x::XConn};
use penrose_ui::{
    bar::widgets::{Text, Widget},
    Context, Result, TextStyle,
};
use std::{
    cmp::max,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

pub const MIN_DURATION: Duration = Duration::from_secs(1);

/// A self-updating [Text] widget that is driven by a [CronRunner].
///
/// See [CronRunner::new_cron_text] for details on how to construct. [CronRunner::run_threaded]
/// _must_ be called before starting window manager executor for the contents of this widget to
/// update.
#[derive(Debug)]
pub struct CronText {
    inner: Arc<Mutex<Text>>,
}

impl CronText {
    /// The current text content of this widget.
    pub fn content(&self) -> String {
        self.inner_guard().get_text().to_string()
    }

    fn inner_guard(&self) -> std::sync::MutexGuard<Text> {
        match self.inner.lock() {
            Ok(inner) => inner,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

impl<X: XConn> Widget<X> for CronText {
    fn draw(&mut self, ctx: &mut Context<'_>, s: usize, f: bool, w: u32, h: u32) -> Result<()> {
        Widget::<X>::draw(&mut *self.inner_guard(), ctx, s, f, w, h)
    }

    fn current_extent(&mut self, ctx: &mut Context<'_>, h: u32) -> Result<(u32, u32)> {
        Widget::<X>::current_extent(&mut *self.inner_guard(), ctx, h)
    }

    fn is_greedy(&self) -> bool {
        Widget::<X>::is_greedy(&*self.inner_guard())
    }

    fn require_draw(&self) -> bool {
        Widget::<X>::require_draw(&*self.inner_guard())
    }
}

#[derive(Debug)]
struct CronHandle {
    next: Instant,
    interval: Duration,
    get_text: fn() -> Option<String>,
    txt: Arc<Mutex<Text>>,
}

impl CronHandle {
    /// Call our `get_text` function to update the contents of our paired [CronText] and then bump
    /// our `next` time to the next interval point.
    ///
    /// This is gives us behaviour of a consistent interval between invocation end/start but not
    /// necessarily a consistent interval between start/start depending on how long `get_text`
    /// takes to run.
    fn update_text(&mut self) {
        let s = (self.get_text)();

        {
            let mut t = match self.txt.lock() {
                Ok(inner) => inner,
                Err(poisoned) => poisoned.into_inner(),
            };
            t.set_text(s.unwrap_or_default());
        }

        let next = self.next + self.interval;
        let now = Instant::now();
        self.next = max(next, now);
    }
}

/// A runner for one or more [CronText] widgets constructed with [CronRunner::new_cron_text].
///
/// New `CronText` widgets will have a corresponding handle stored within the runner that
/// constructed them. You must call [CronRunner::run_threaded] in order to actually update the
/// contents of the widgets.
#[derive(Debug, Default)]
pub struct CronRunner {
    inner: Vec<CronHandle>,
}

impl CronRunner {
    /// Register a new [CronText] with this runner for later updating via
    /// [CronRunner::run_threaded].
    ///
    /// This method will panic if the provided `interval` is less than [MIN_DURATION].
    pub fn new_cron_text(
        &mut self,
        style: TextStyle,
        get_text: fn() -> Option<String>,
        interval: Duration,
    ) -> CronText {
        if interval < MIN_DURATION {
            panic!("CronText interval is too small: {interval:?} < {MIN_DURATION:?}");
        }

        let h = CronHandle {
            next: Instant::now(),
            interval,
            get_text,
            txt: Arc::new(Mutex::new(Text::new("?", style, false, false))),
        };

        let inner = h.txt.clone();
        self.inner.push(h);

        CronText { inner }
    }

    /// Run the polling thread for all registered [CronText] widgets and update their contents on
    /// their requested intervals.
    pub fn run_threaded(mut self) {
        thread::spawn(move || loop {
            while self.inner[0].next < Instant::now() {
                self.inner[0].update_text();
                self.inner.sort_by(|a, b| a.next.cmp(&b.next));
            }

            // FIXME: this is a hack at the moment to ensure that an event drops into the main
            // window manager event loop and triggers the `on_event` hook of the status bar.
            let _ = spawn_with_args("xsetroot", &["-name", ""]);

            let interval = self.inner[0].next - Instant::now();
            thread::sleep(interval);
        });
    }
}
