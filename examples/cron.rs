//! Testing out the cron statusbar widget behaviour
use penrose_sminez::{
    bar::{amixer_text, battery_text, date_text, weather_text, wifi_text},
    schedule::CronRunner,
    BLACK, WHITE,
};
use penrose_ui::TextStyle;
use std::{fmt::Write, thread, time::Duration};

fn main() {
    let mut widgets = Vec::new();
    let mut runner = CronRunner::default();

    let style = TextStyle {
        fg: WHITE.into(),
        bg: Some(BLACK.into()),
        padding: (2, 2),
    };

    let ms = |n: u64| Duration::from_millis(n);

    widgets.push(runner.new_cron_text(style, weather_text, ms(300_000)));
    widgets.push(runner.new_cron_text(style, wifi_text, ms(1000)));
    widgets.push(runner.new_cron_text(style, || battery_text("BAT1"), ms(1200)));
    widgets.push(runner.new_cron_text(style, || amixer_text("Master"), ms(1400)));
    widgets.push(runner.new_cron_text(style, date_text, ms(10_000)));

    runner.run_threaded();

    loop {
        let s: String = widgets.iter().fold(String::new(), |mut s, w| {
            let _ = write!(s, " {}", w.content());
            s
        });

        println!("{s}");
        thread::sleep(Duration::from_millis(1000));
    }
}
