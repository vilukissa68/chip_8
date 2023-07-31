use crate::CPU;
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    buffer::Buffer,
    style::{Color, Style},
    text::Span,
    widgets::{
        canvas::{Canvas, Map, MapResolution, Rectangle, Context, Painter},
        Block, Borders, Row, Table, Cell
    },
    Frame, Terminal, symbols,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use resize::{Pixel::RGB8, px::RGB};
use resize::Type::Point;

struct Tui {
    cpu: CPU,
    width: u8,
    height: u8,
    keys: [bool; 16],
}

impl Tui {
    fn new() -> Tui {
        Tui {
            cpu: CPU::new(),
            width: 64,
            height: 32,
            keys: [false; 16],
        }
    }

    fn on_tick(&mut self) {
        return
    }
}

pub fn tui_start() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(50);
    let tui = Tui::new();
    let res = run_tui(&mut terminal, tui, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}


fn run_tui<B: Backend>(
    terminal: &mut Terminal<B>,
    mut tui: Tui,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &tui))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                }

            }
        }

        if last_tick.elapsed() >= tick_rate {
            tui.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, tui: &Tui) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());
    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("World"))
        .paint(|ctx| {
            ctx.draw(&Map {
                color: Color::White,
                resolution: MapResolution::High,
            });
            ctx.print(
                0.0,
                0.0,
                Span::styled("You are here", Style::default().fg(Color::Yellow)),
            );
        })
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0]);
    f.render_widget(canvas, chunks[0]);

    // Map cpu vbuf to canvas
    let mut pixels = Vec::new();
    for y in 0..32 {
        for x in 0..64 {
            if tui.cpu.read_vbuf(x, y) {
                pixels.push(&true);
            } else {
                pixels.push(&false);
            }
        }
    }
    let fb = FrameBuffer::new(pixels)
        .block(Block::default().borders(Borders::ALL).title("Display"));
    f.render_widget(fb, chunks[1]);
}

// Framebuffer object from CPU

#[derive(Default)]
struct FrameBuffer<'a> {
    pixels: Vec<&'a bool>,
    block: Option<Block<'a>>,
}

impl<'a> FrameBuffer<'a> {
    fn new(pixels: Vec<&'a bool>) -> Self {
        Self { pixels, ..Default::default() }
    }

    fn block(mut self, block: Block<'a>) -> FrameBuffer<'a> {
        self.block = Some(block);
        self
    }
}
impl tui::widgets::Widget for FrameBuffer<'_> {
    fn render(mut self, area: Rect, buf: &mut tui::buffer::Buffer) {
        let area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        let mut resizer = resize::new(
            64,
            32,
            area.width as usize,
            area.height as usize,
            RGB8,
            Point,
        ).unwrap();

        let mut dst = vec![RGB::new(0, 0, 0); (area.width * area.height) as usize];

        // Construct the framebuffer
        let mut fb = Vec::new();
        for j in 0..32 {
             for i in 0..64 {
                 if *self.pixels[i as usize + j * 32 as usize] {
                     fb.push(RGB::new(255, 255, 255));
                 } else {
                     fb.push(RGB::new(0, 0, 0));
                 }
             }
         }

        resizer.resize(&fb, &mut dst).unwrap();
        let mut dst = dst.iter();
            for j in area.y..area.y + area.height {
                for i in area.x..area.x + area.width {
                    let rgb = dst.next().unwrap();
                    let r = rgb.r;
                    let g = rgb.g;
                    let b = rgb.b;
                    buf.get_mut(i, j).set_bg(Color::Rgb(r, g, b));
                }
            }
    }
}
