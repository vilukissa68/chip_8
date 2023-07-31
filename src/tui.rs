use crate::CPU;
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    widgets::{
        canvas::{Canvas, Map, MapResolution, Rectangle, Context, Painter},
        Block, Borders, Row, Table, Cell, TableState
    },
    Frame, Terminal, symbols,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use resize::{Pixel::RGB8, px::RGB};

struct Tui {
    cpu: CPU,
    width: u8,
    height: u8,
    keys: [bool; 16],
    executing: bool,
    register_table_state: TableState,
}

impl Tui {
    fn new(debug: bool) -> Tui {
        Tui {
            cpu: CPU::new(debug),
            width: 64,
            height: 32,
            keys: [false; 16],
            executing: false,
            register_table_state: TableState::default(),
        }
    }

    fn on_tick(&mut self) {
        if self.executing {
            if self.cpu.next_cycle() == -1 {
                self.executing = false; // Program ended
            }
        }
    }
}

pub fn tui_start(binary: Vec<u8>, debug: bool) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let mut tui = Tui::new(debug);
    tui.cpu.load_bin(binary, false);
    tui.executing = true;
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
        terminal.draw(|f| ui(f, &mut tui))?;

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

fn register_view(registers: Vec<u8>) -> Table<'static> {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["Register", "Value"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

    let rows = registers.into_iter().enumerate().map(|(idx, reg)| {
        let height = 1;
        let cells = [Cell::from(format!("V{:X}",idx)), Cell::from(format!("{:X}", reg))];
        Row::new(cells).height(height as u16).bottom_margin(1)
    });

    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Table"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(10),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);

    return t;
}

fn ui<B: Backend>(f: &mut Frame<B>, tui: &mut Tui) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let data_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);

    // Registers
    let register_view = register_view(tui.cpu.get_registers().to_vec());
    f.render_stateful_widget(register_view, data_chunks[0], &mut tui.register_table_state);


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
            resize::Type::Point,
        ).unwrap();

        let mut dst = vec![RGB::new(0, 0, 0); (area.width * area.height) as usize];

        // Construct the framebuffer
        let mut fb = Vec::new();
        for j in 0..32 {
             for i in 0..64 {
                 if *self.pixels[i as usize + j * 64 as usize] {
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
