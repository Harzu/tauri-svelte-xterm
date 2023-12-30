use alacritty_terminal::{
    vte::ansi::{self, Processor},
    event::{EventListener, OnResize, WindowSize},
    term::{test::TermSize, cell::Flags},
    tty::{EventedReadWrite, Pty},
    Term,
};
use std::{
    fs::File,
    io::Write,
};

use crate::font;

#[derive(Clone)]
pub struct EventProxy;

impl EventProxy {}

impl EventListener for EventProxy {
    fn send_event(&self, _: alacritty_terminal::event::Event) {}
}

struct InnerTerminalSize {
    cols: u16,
    rows: u16,
}

pub struct Terminal {
    tty: Pty,
    term: Term<EventProxy>,
    parser: Processor,
    size: InnerTerminalSize,
}

impl Terminal {
    pub fn new(shell: String) -> Self {
        let mut pty_config = alacritty_terminal::tty::Options::default();
        pty_config.shell = Some(alacritty_terminal::tty::Shell::new(shell, vec![]));

        let config = alacritty_terminal::term::Config::default();
        let inner_size = InnerTerminalSize { cols: 100, rows: 50 };
        let size = WindowSize {
            cell_width: 1,
            cell_height: 1,
            num_cols: inner_size.cols,
            num_lines: inner_size.rows,
        };
        let term_size = TermSize::new(100, 50);
        let event_proxy = EventProxy {};
        let tty = alacritty_terminal::tty::new(&pty_config, size, 0).unwrap();
        let term = alacritty_terminal::Term::new(config, &term_size, event_proxy);
        let parser = ansi::Processor::new();

        Self { tty, term, parser, size: inner_size }
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        if rows != self.size.rows || cols != self.size.cols {
            let size = WindowSize {
                cell_width: 1,
                cell_height: 1,
                num_cols: cols,
                num_lines: rows,
            };
    
            self.tty.on_resize(size);
            self.term.resize(TermSize::new(
                size.num_cols as usize,
                size.num_lines as usize,
            ));

            self.size.cols = cols;
            self.size.rows = rows;
        }
    }

    pub fn new_reader(&mut self) -> File {
        self.tty.reader().try_clone().unwrap()
    }

    pub fn update(&mut self, data: Vec<u8>) {
        for item in data.to_vec() {
            self.parser.advance(&mut self.term, item);
        }
    }

    pub fn write_to_pty(&mut self, c: char) {
        self.tty.writer().write_all(&[c as u8]).unwrap();
    }

    pub fn cells(&self) -> Vec<Cell> {
        let mut res = vec![];
        let content = self.term.renderable_content();
        for item in content.display_iter {
            let point = item.point;
            let cell = item.cell;

            let mut fg = font::get_color(cell.fg);
            let mut bg = font::get_color(cell.bg);

            if cell.flags.contains(Flags::DIM) || cell.flags.contains(Flags::DIM_BOLD) {
                fg = font::Color::from_rgba(fg.r, fg.g, fg.b, 66);
            }

            let inverse = cell.flags.contains(Flags::INVERSE);
            if inverse {
                let clone_fg = fg.clone();
                fg = bg;
                bg = clone_fg;
            }

            res.push(Cell {
                column: point.column.0,
                line: point.line.0,
                content: cell.c,
                display_offset: content.display_offset,
                fg: fg,
                bg: bg,
            })
        }

        res
    }
}

#[derive(Clone, serde::Serialize, Debug)]
pub struct Cell {
    pub column: usize,
    pub line: i32,
    pub content: char,
    pub display_offset: usize,
    pub fg: font::Color,
    pub bg: font::Color,
}
