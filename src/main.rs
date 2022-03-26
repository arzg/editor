use crossterm::{cursor, event, queue, terminal};
use std::io::{self, Write};
use std::{env, fs};

fn main() -> io::Result<()> {
    let text = match env::args().nth(1) {
        Some(file_to_edit) => fs::read_to_string(file_to_edit)?,
        None => String::new(),
    };
    let stdout = io::stdout();

    Editor::new(text, stdout.lock())?.run()?;

    Ok(())
}

#[derive(Debug)]
struct Editor<'a> {
    buffer: Vec<String>,
    stdout: io::StdoutLock<'a>,
    width: usize,
    height: usize,
    row: usize,
    column: usize,
    scroll: usize,
    should_exit: bool,
}

impl<'a> Editor<'a> {
    fn new(buffer: String, stdout: io::StdoutLock<'a>) -> io::Result<Self> {
        let (width, height) = terminal::size()?;

        Ok(Self {
            buffer: buffer.split('\n').map(str::to_string).collect(),
            stdout,
            width: width.into(),
            height: height.into(),
            row: 0,
            column: 0,
            scroll: 0,
            should_exit: false,
        })
    }

    fn run(mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;

        while !self.should_exit {
            self.render()?;
            self.handle_event()?;
        }

        terminal::disable_raw_mode()?;

        Ok(())
    }

    fn render(&mut self) -> io::Result<()> {
        queue!(
            self.stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        for (idx, line) in self
            .buffer
            .iter()
            .skip(self.scroll)
            .take(self.height)
            .enumerate()
        {
            let line = if line.len() < self.width {
                line
            } else {
                &line[..self.width]
            };

            write!(self.stdout, "\r{}", line)?;
            if idx != self.height - 1 {
                writeln!(self.stdout)?;
            }
        }

        queue!(
            self.stdout,
            cursor::MoveTo(self.column as u16, (self.row - self.scroll) as u16)
        )?;

        self.stdout.flush()?;

        Ok(())
    }

    fn handle_event(&mut self) -> io::Result<()> {
        match event::read()? {
            event::Event::Key(key) => match key {
                event::KeyEvent {
                    code,
                    modifiers: event::KeyModifiers::NONE,
                } => match code {
                    event::KeyCode::Backspace => self.backspace(),
                    event::KeyCode::Enter => self.enter(),
                    event::KeyCode::Left => self.left(),
                    event::KeyCode::Right => self.right(),
                    event::KeyCode::Up => self.up(),
                    event::KeyCode::Down => self.down(),
                    event::KeyCode::Home => self.column = 0,
                    event::KeyCode::End => self.column = self.buffer[self.row].len(),
                    event::KeyCode::PageUp => todo!(),
                    event::KeyCode::PageDown => todo!(),
                    event::KeyCode::Tab => todo!(),
                    event::KeyCode::BackTab => todo!(),
                    event::KeyCode::Delete => todo!(),
                    event::KeyCode::Insert => todo!(),
                    event::KeyCode::F(_) => todo!(),
                    event::KeyCode::Char(c) => self.keypress(c),
                    event::KeyCode::Null => {}
                    event::KeyCode::Esc => self.should_exit = true,
                },
                event::KeyEvent { .. } => {}
            },
            event::Event::Mouse(_) => {}
            event::Event::Resize(width, height) => {
                self.width = width.into();
                self.height = height.into();
            }
        }

        self.scroll_to_show_cursor();

        std::net::TcpStream::connect("127.0.0.1:9292")
            .unwrap()
            .write_all(format!("\n\n\n\n\n\n\n\n{self:#?}").as_bytes())
            .unwrap();

        Ok(())
    }

    fn scroll_to_show_cursor(&mut self) {
        let top_line = self.scroll;
        let bottom_line = self.scroll + self.height;

        if self.row < top_line {
            self.scroll = self.row;
        } else if self.row >= bottom_line {
            self.scroll = self.row - self.height + 1;
        }
    }

    fn keypress(&mut self, c: char) {
        self.buffer[self.row].insert(self.column, c);
        self.column += 1;
    }

    fn backspace(&mut self) {
        if self.column == 0 {
            if self.row == 0 {
                return;
            }

            let row = self.buffer.remove(self.row);
            self.row -= 1;
            let len = self.buffer[self.row].len();
            self.buffer[self.row].push_str(&row);
            self.column = len;
            return;
        }

        self.column -= 1;
        self.buffer[self.row].remove(self.column);
    }

    fn enter(&mut self) {
        let rest = self.buffer[self.row].split_off(self.column);
        self.row += 1;
        self.buffer.insert(self.row, rest);
        self.column = 0;
    }

    fn left(&mut self) {
        if self.column != 0 {
            self.column -= 1;
        }
    }
    fn right(&mut self) {
        if self.column < self.buffer[self.row].len() {
            self.column += 1;
        }
    }
    fn up(&mut self) {
        if self.row != 0 {
            self.row -= 1;
        }
        self.clamp_column();
    }
    fn down(&mut self) {
        if self.row < self.buffer.len() - 1 {
            self.row += 1;
        }
        self.clamp_column();
    }

    fn clamp_column(&mut self) {
        let len = self.buffer[self.row].len();
        if self.column > len {
            self.column = len;
        }
    }
}
