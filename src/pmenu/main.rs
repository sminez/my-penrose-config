//! A penrose Draw backed implementation of dmenu
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use penrose::{
    core::{
        bindings::KeyPress,
        data_types::{Region, WinId, WinType},
        xconnection::Atom,
    },
    draw::{
        Color, DrawContext, DrawError, KeyPressDraw, KeyPressResult, Result, Text, TextStyle,
        Widget,
    },
    xcb::XcbDraw,
};

use std::io::{self, Read};

const BLACK: u32 = 0x282828ff;
const WHITE: u32 = 0xebdbb2ff;
const BLUE: u32 = 0x458588ff;
const PROFONT: &str = "ProFont For Powerline";

pub enum PMenuMatch {
    Line(usize, String),
    UserInput(String),
    NoMatch,
}

struct TextWithSelection {
    lines: Vec<String>,
    selected: usize,
    max_lines: usize,
    font: String,
    point_size: i32,
    padding: f64,
    bg: Color,
    fg: Color,
    fg_sel: Color,
    bg_sel: Color,
    require_draw: bool,
    extent: Option<(f64, f64)>,
}

impl TextWithSelection {
    fn new(
        font: String,
        point_size: i32,
        padding: f64,
        bg: Color,
        fg: Color,
        bg_sel: Color,
        fg_sel: Color,
    ) -> Self {
        Self {
            lines: vec![],
            selected: 0,
            max_lines: 10,
            font,
            point_size,
            padding,
            bg,
            fg,
            bg_sel,
            fg_sel,
            require_draw: false,
            extent: None,
        }
    }

    fn set_input(&mut self, lines: Vec<String>, selected: usize) {
        self.lines = lines;
        self.selected = selected;
        self.require_draw = true;
    }

    fn set_max_lines(&mut self, max_lines: usize) {
        self.max_lines = max_lines;
    }
}

impl Widget for TextWithSelection {
    fn draw(
        &mut self,
        ctx: &mut dyn DrawContext,
        _screen: usize,
        _screen_has_focus: bool,
        w: f64,
        h: f64,
    ) -> Result<()> {
        ctx.color(&self.bg);
        ctx.rectangle(0.0, 0.0, w, h);
        ctx.font(&self.font, self.point_size)?;

        for (ix, line) in self.lines.iter().enumerate() {
            let (lw, lh) = ctx.text_extent(line)?;
            let fg = if ix == self.selected {
                ctx.color(&self.bg_sel);
                ctx.rectangle(0.0, 0.0, lw + self.padding * 2.0, lh);
                self.fg_sel
            } else {
                self.fg
            };

            ctx.color(&fg);
            ctx.text(line, 0.0, (self.padding, self.padding))?;
            ctx.translate(0.0, lh);
        }

        Ok(())
    }

    fn current_extent(&mut self, ctx: &mut dyn DrawContext, _h: f64) -> Result<(f64, f64)> {
        match self.extent {
            Some(extent) => Ok(extent),
            None => {
                let mut height = 0.0;
                let mut w_max = 0.0;
                for line in self.lines.iter() {
                    ctx.font(&self.font, self.point_size)?;
                    let (w, h) = ctx.text_extent(line)?;
                    height += h;
                    w_max = if w > w_max { w } else { w_max };
                }

                let ext = (w_max + self.padding, height);
                self.extent = Some(ext);
                Ok(ext)
            }
        }
    }

    fn require_draw(&self) -> bool {
        self.require_draw
    }

    fn is_greedy(&self) -> bool {
        false
    }
}

pub struct PMenu<D>
where
    D: KeyPressDraw,
{
    drw: D,
    id: Option<WinId>,
    bg: Color,
    prompt: Text,
    patt: Text,
    txt: TextWithSelection,
    w: f64,
    h: f64,
}

impl<D> PMenu<D>
where
    D: KeyPressDraw,
{
    pub fn new(
        mut drw: D,
        fg: impl Into<Color>,
        bg: impl Into<Color>,
        hl: impl Into<Color>,
        font: impl Into<String>,
        point_size: i32,
    ) -> Result<Self> {
        let font = font.into();
        let bg = bg.into();
        let fg = fg.into();
        let hl = hl.into();

        drw.register_font(&font);

        let default_style = TextStyle {
            font: font.clone(),
            point_size,
            fg,
            bg: Some(bg),
            padding: (1.0, 1.0),
        };

        Ok(Self {
            drw,
            bg,
            txt: TextWithSelection::new(font, point_size, 3.0, bg, fg, hl, fg),
            patt: Text::new("", &default_style, false, true),
            prompt: Text::new("", &default_style, false, true),
            w: 0.0,
            h: 0.0,
            id: None,
        })
    }

    fn init_window(&mut self, screen_index: usize, w_max: f64, h_max: f64) -> Result<()> {
        if !(0.0..=1.0).contains(&w_max) || !(0.0..=1.0).contains(&h_max) {
            return Err(DrawError::Raw(format!(
                "w_max and h_max must be in the range 0.0..1.0: w_max={}, h_max={}",
                w_max, h_max
            )));
        }

        let screen_region = self
            .drw
            .screen_sizes()?
            .get(screen_index)
            .ok_or_else(|| DrawError::Raw("screen_index out of range".into()))?
            .clone();

        let r = screen_region
            .scale_w(w_max)
            .scale_h(h_max)
            .centered_in(&screen_region)
            .unwrap(); // We know we are bounded by screen_region

        let (x, y, w, h) = r.values();
        self.w = w as f64;
        self.h = h as f64;

        let mut id =
            self.drw
                .new_window(WinType::InputOutput(Atom::NetWindowTypeDialog), r, true)?;

        let mut ctx = self.drw.context_for(id)?;
        let (prompt_w, prompt_h) = self.prompt.current_extent(&mut ctx, 1.0)?;
        let (input_w, input_h) = self.txt.current_extent(&mut ctx, 1.0)?;
        let w = (prompt_w + input_w) * 1.1;
        let h = (prompt_h + input_h) * 1.1;

        if r.contains(&Region::new(x, y, w as u32, h as u32)) {
            self.drw.destroy_window(id);
            self.drw.flush(id);
            self.w = w;
            self.h = h;

            id = self.drw.new_window(
                WinType::InputOutput(Atom::NetWindowTypeDialog),
                Region::new(0, 0, w as u32, h as u32)
                    .centered_in(&screen_region)
                    .unwrap(),
                true,
            )?;
        }

        self.drw.flush(id);
        self.id = Some(id);

        Ok(())
    }

    fn redraw(&mut self, patt: &str, matches: &[(usize, &String)], selected: usize) -> Result<()> {
        self.patt.set_text(patt);
        let lines = matches.iter().map(|(_, line)| line.to_string()).collect();
        self.txt.set_input(lines, selected);

        let id = self.id.unwrap();
        let mut ctx = self.drw.context_for(id)?;

        ctx.clear();
        ctx.color(&self.bg);
        ctx.rectangle(0.0, 0.0, self.w, self.h);

        let (w, h) = self.prompt.current_extent(&mut ctx, self.h)?;
        self.prompt.draw(&mut ctx, 0, false, w, h)?;
        ctx.translate(w, 0.0);

        self.patt.draw(&mut ctx, 0, false, w, h)?;
        ctx.translate(0.0, h);

        self.txt.draw(&mut ctx, 0, true, w, h)?;

        ctx.flush();
        self.drw.flush(id);
        Ok(())
    }

    pub fn get_selection_from_input(
        &mut self,
        prompt: impl Into<String>,
        input: Vec<impl Into<String>>,
        max_lines: usize,
        screen_index: usize,
        w_max: f64,
        h_max: f64,
    ) -> Result<PMenuMatch> {
        let input: Vec<String> = input.into_iter().map(|s| s.into()).collect();
        self.prompt.set_text(prompt);
        self.txt.set_input(input.clone(), 0);
        self.txt.set_max_lines(if max_lines < input.len() {
            max_lines
        } else {
            input.len()
        });

        self.init_window(screen_index, w_max, h_max)?;
        let selection = self.get_selection_inner(input);
        self.drw.destroy_window(self.id.unwrap());
        self.id = None;

        selection
    }

    fn get_selection_inner(&mut self, input: Vec<String>) -> Result<PMenuMatch> {
        let mut patt = String::new();
        let mut matches: Vec<(usize, &String)> = input.iter().enumerate().collect();
        let mut selected = 0;

        let matcher = SkimMatcherV2::default();

        loop {
            if let KeyPressResult::KeyPress(k) = self.drw.next_keypress() {
                match k {
                    KeyPress::Return if selected < matches.len() => {
                        let m = matches[selected];
                        return Ok(PMenuMatch::Line(m.0, m.1.clone()));
                    }

                    KeyPress::Escape | KeyPress::Return => {
                        return if patt.is_empty() {
                            Ok(PMenuMatch::NoMatch)
                        } else {
                            Ok(PMenuMatch::UserInput(patt))
                        };
                    }

                    KeyPress::Backspace => {
                        if patt.len() > 0 {
                            patt.pop();
                        }
                    }

                    KeyPress::Up => {
                        if selected > 0 {
                            selected -= 1
                        }
                    }
                    KeyPress::Down => {
                        if !matches.is_empty() && selected < matches.len() - 1 {
                            selected += 1
                        }
                    }

                    KeyPress::Utf8(c) => {
                        patt.push_str(&c);
                        selected = 0;
                    }

                    _ => continue,
                };
            }

            let mut scored = input
                .iter()
                .enumerate()
                .flat_map(|(i, line)| {
                    matcher
                        .fuzzy_match(line, &patt)
                        .map(|score| (score, (i, line)))
                })
                .collect::<Vec<_>>();

            scored.sort_by_key(|(score, _)| -*score);
            matches = scored.into_iter().map(|(_, data)| data).collect();
            self.redraw(&patt, &matches, selected)?;
        }
    }
}

fn main() -> Result<()> {
    let mut buffer = String::new();
    let mut stdin = io::stdin();
    stdin.read_to_string(&mut buffer).unwrap();
    let lines = buffer.trim().split('\n').map(|s| s.to_string()).collect();

    let drw = XcbDraw::new()?;
    let mut p = PMenu::new(drw, WHITE, BLACK, BLUE, PROFONT, 12)?;

    match p.get_selection_from_input("what would you like to do?", lines, 10, 0, 0.8, 0.8)? {
        PMenuMatch::Line(i, s) => println!("matched {} on line {}", s, i),
        PMenuMatch::UserInput(s) => println!("user input: {}", s),
        PMenuMatch::NoMatch => println!("no match"),
    }

    Ok(())
}
