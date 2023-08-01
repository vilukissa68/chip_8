use crate::CPU;
use crate::disassembler::{decode};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    widgets::{
        Block, Borders, Row, Table, Cell, TableState, Paragraph, List, ListItem, ListState
    },
    text::{Span, Spans},

    Frame, Terminal, symbols,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use resize::{Pixel::RGB8, px::RGB};

enum Window {
    Memory,
    Registers,
    Instructions,
    //CPU,
}

struct Tui {
    cpu: CPU,
    width: u8,
    height: u8,
    keys: [bool; 16],
    executing: bool,
    current_window: Window,
    register_table_state: TableState,
    memory_table_state: TableState,
    cpu_table_state: TableState,
    instruction_list_state: ListState,
}

impl Tui {
    fn new(debug: bool) -> Tui {
        Tui {
            cpu: CPU::new(debug),
            width: 64,
            height: 32,
            keys: [false; 16],
            executing: false,
            current_window: Window::Memory,
            register_table_state: TableState::default(),
            memory_table_state: TableState::default(),
            cpu_table_state: TableState::default(),
            instruction_list_state: ListState::default(),
        }
    }
    fn next_cycle(&mut self) {
        self.cpu.next_cycle();
    }

    fn on_tick(&mut self) {
        if self.executing {
            if self.cpu.next_cycle() == -1 {
                self.executing = false; // Program ended
            }
        }
    }

    pub fn cycle_window(&mut self) {
        match self.current_window {
            Window::Memory => self.current_window = Window::Registers,
            Window::Registers => self.current_window = Window::Instructions,
            Window::Instructions => self.current_window = Window::Memory,

        }
    }

    pub fn handle_next_table(&mut self) {
        let (func, state) = match self.current_window {
            Window::Memory => (self.cpu.get_memory().len() / 16, &mut self.memory_table_state),
            Window::Registers => (self.cpu.get_registers().len(), &mut self.register_table_state),
            _ => panic!("Invalid window"),
        };

        let i = match state.selected() {
            Some(i) => {
                if i >= func - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        state.select(Some(i));
    }

    pub fn handle_prev_table(&mut self) {
        let (func, state) = match self.current_window {
            Window::Memory => (self.cpu.get_memory().len() / 16, &mut self.memory_table_state),
            Window::Registers => (self.cpu.get_registers().len(), &mut self.register_table_state),
            _ => panic!("Invalid window"),
        };

        let i = match state.selected() {
            Some(i) => {
                if i == 0 {
                    func - 1
                } else {
                    i - 1
                }
            }
            None => func - 1,
        };
        state.select(Some(i));
    }

    pub fn handle_next_list(&mut self) {
        let (func, state) = match self.current_window {
            Window::Instructions => (self.cpu.get_history().len() + 1, &mut self.instruction_list_state),
            _ => panic!("Invalid window"),
        };

        let i = match state.selected() {
            Some(i) => {
                if i >= func - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        state.select(Some(i));
    }

    pub fn handle_prev_list(&mut self) {
        let (func, state) = match self.current_window {
            Window::Instructions => (self.cpu.get_history().len() + 1, &mut self.instruction_list_state),
            _ => panic!("Invalid window"),
        };

        let i = match state.selected() {
            Some(i) => {
                if i == 0 {
                    func - 1
                } else {
                    i - 1
                }
            }
            None => func - 1,
        };
        state.select(Some(i));
    }

    pub fn handle_next(&mut self) {
        match self.current_window {
            Window::Memory | Window::Registers => self.handle_next_table(),
            Window::Instructions => self.handle_next_list(),
        }
    }

    pub fn handle_prev(&mut self) {
        match self.current_window {
            Window::Memory | Window::Registers => self.handle_prev_table(),
            Window::Instructions => self.handle_prev_list(),
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
                    KeyCode::Down => tui.handle_next(),
                    KeyCode::Up => tui.handle_prev(),
                    KeyCode::Tab => tui.cycle_window(),
                    KeyCode::Char('p') => tui.executing = !tui.executing,
                    KeyCode::Char('n') => tui.next_cycle(),
                    KeyCode::Char('q') => {return Ok(());}
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

fn cpu_view(tui: &Tui) -> Table<'static> {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD);
    let normal_style = Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD);
    let text_style = Style::default().bg(Color::Reset).add_modifier(Modifier::BOLD);
    let header_cells = ["Register", "Value"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(0);

    // Take 16 at a time
    let mut rows = Vec::new();
    rows.push(Row::new(vec![Cell::from("PC:"), Cell::from(format!("{:X}", tui.cpu.pc))]).bottom_margin(1).style(text_style));
    rows.push(Row::new(vec![Cell::from("I:"), Cell::from(format!("{:X}", tui.cpu.ir))]).bottom_margin(1).style(text_style));
    rows.push(Row::new(vec![Cell::from("SP:"), Cell::from(format!("{:X}", tui.cpu.sp))]).bottom_margin(1).style(text_style));
    rows.push(Row::new(vec![Cell::from("DT:"), Cell::from(format!("{:X}", tui.cpu.dt))]).bottom_margin(1).style(text_style));
    rows.push(Row::new(vec![Cell::from("ST:"), Cell::from(format!("{:X}", tui.cpu.st))]).bottom_margin(1).style(text_style));
    if tui.executing {
        rows.push(Row::new(vec![Cell::from("Running:"), Cell::from("")]).bottom_margin(1).style(text_style));
    } else {
        rows.push(Row::new(vec![Cell::from("Paused:"), Cell::from("")]).bottom_margin(1).style(text_style));
    }


    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("CPU"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Percentage(30),
                    ]);
    return t;
}

fn register_view(tui: &Tui) -> Table<'static> {
    let registers = tui.cpu.get_registers();
    let selected_style = Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD);
    let normal_style = Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD);
    let text_style = Style::default().bg(Color::Reset).add_modifier(Modifier::BOLD);
    let header_cells = ["Register", "Value"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(0);

    let rows = registers.into_iter().enumerate().map(|(idx, reg)| {
        let height = 1;
        let cells = [Cell::from(format!("V{:X}:",idx)), Cell::from(format!("{:X}", reg))];
        Row::new(cells).height(height as u16).bottom_margin(0).style(text_style)
    });

    let border_style = match tui.current_window {
        Window::Registers => Style::default().fg(Color::Yellow),
        _ => Style::default(),
    };

    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Registers").border_style(border_style))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ]);

    return t;
}

fn memory_view(tui: &Tui) -> Table<'static> {
    let memory = tui.cpu.get_memory();
    let selected_style = Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD);
    let normal_style = Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD);
    let text_style = Style::default().bg(Color::Reset).add_modifier(Modifier::BOLD);

    let header_cells = ["Address", "0x00", "0x01", "0x02", "0x03", "0x04", "0x05", "0x06",
                        "0x07", "0x08", "0x09", "0x0A", "0x0B", "0x0C", "0x0D", "0x0E", "0x0F"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(0);

    // Take 16 at a time
    let mut rows = Vec::new();
    let mut cells = Vec::new();
    for (idx, x) in memory.into_iter().enumerate() {
        if idx % 16 == 0 {
            if idx != 0 {
                let height = x.to_string().chars().filter(|c| *c == '\n').count() + 1;
                rows.push(Row::new(cells).height(height as u16).bottom_margin(0).style(text_style));
                cells = Vec::new();
            }
            cells.push(Cell::from(format!("{:X}", idx))); // Address
        }

        cells.push(Cell::from(format!("{:01$x}", x, 2))); // Value in address
    }
    // Push last row
    rows.push(Row::new(cells).height(1).bottom_margin(1));

    let border_style = match tui.current_window {
        Window::Memory => Style::default().fg(Color::Yellow),
        _ => Style::default(),
    };


    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("RAM").border_style(border_style))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(7),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(5),
                    ]);
    return t;
}


fn instruction_view(tui: &Tui) -> List<'static> {
    let mut items: Vec<ListItem> = Vec::new();
    let next_inst = tui.cpu.fetch_no_increment();
    let next_line = Spans::from(Span::styled(
        format!("Next:  {:6X} | {}", next_inst, decode(next_inst)),
        Style::default().add_modifier(Modifier::BOLD),
    ));
    let next_item = ListItem::new(next_line).style(Style::default().fg(Color::Black).bg(Color::Blue));
    items.push(next_item);

    let history: Vec<ListItem> = tui.cpu.get_history()
                                        .iter()
                                        .enumerate()
                                        .rev()
                                        .map(|(idx, i)| {
                                            let hex = format!("{:01$x}", i,4);
                                            let line =  Spans::from(Span::styled(
                                                format!("{:5}  {} | {}", idx, hex, decode(*i)),
                                                Style::default().add_modifier(Modifier::BOLD),
                                            ));
                                            ListItem::new(line).style(Style::default().fg(Color::White).bg(Color::Reset))
                                        })
                                        .collect();

    items.extend(history);
    let border_style = match tui.current_window {
        Window::Instructions => Style::default().fg(Color::Yellow),
        _ => Style::default(),
    };


    // Create a List from all list items and highlight the currently selected one
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List").border_style(border_style))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    return list;
}

fn ui<B: Backend>(f: &mut Frame<B>, tui: &mut Tui) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let data_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(45), Constraint::Percentage(10)].as_ref())
        .split(chunks[0]);

    let data_chunks_upper = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(34), Constraint::Percentage(33)].as_ref())
        .split(data_chunks[0]);


    // CPU
    let cpu_view = cpu_view(tui);
    f.render_stateful_widget(cpu_view, data_chunks_upper[0], &mut tui.cpu_table_state);

    // Registers
    let register_view = register_view(tui);
    f.render_stateful_widget(register_view, data_chunks_upper[1], &mut tui.register_table_state);

    // Instruction view
    let instruction_view = instruction_view(tui);
    f.render_stateful_widget(instruction_view, data_chunks_upper[2], &mut tui.instruction_list_state);


    // Memory view
    let memory_view = memory_view(tui);
    f.render_stateful_widget(memory_view, data_chunks[1], &mut tui.memory_table_state);


    // Hep
    let text = vec![
        Spans::from("<TAB> Switch window"),
        Spans::from("<N> Step"),
        Spans::from("<P> Pause/Run"),
];
    let help = Paragraph::new(text.clone())
        .style(Style::default().bg(Color::Reset).fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Left);
    f.render_widget(help, data_chunks[2]);


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

        let rgb_white = Color::White;

        // Construct the framebuffer
        let mut fb: Vec<RGB<u8>> = Vec::new();
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
                    match rgb {
                        RGB { r: 0, g: 0, b: 0 } => buf.get_mut(i, j).set_bg(Color::Reset),
                        RGB { r: 255, g: 255, b: 255 } => buf.get_mut(i, j).set_bg(Color::Indexed(15)),
                        _ => buf.get_mut(i, j).set_bg(Color::Rgb(r, g, b)),
                    };
                }
            }
    }
}
