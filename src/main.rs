use crossterm::style::Stylize;
use crossterm::{cursor, event, queue, style, terminal};
use std::io::{self, Write};
use std::path::PathBuf;
use std::{env, fs};

fn main() -> io::Result<()> {
    let (path, text) = match env::args().nth(1) {
        Some(file_to_edit) => {
            let file_to_edit = PathBuf::from(file_to_edit);
            let text = fs::read_to_string(&file_to_edit)?;
            (Some(file_to_edit), text)
        }
        None => (None, String::new()),
    };
    let stdout = io::stdout();

    Ui::new(text, path, stdout.lock())?.run()?;

    Ok(())
}

#[derive(Debug)]
struct Ui<'a> {
    source_editor: SourceEditor,
    file: Option<PathBuf>,
    stdout: io::StdoutLock<'a>,
    width: usize,
    height: usize,
    should_exit: bool,
}

impl<'a> Ui<'a> {
    fn new(buffer: String, file: Option<PathBuf>, stdout: io::StdoutLock<'a>) -> io::Result<Self> {
        let (width, height) = terminal::size()?;
        let width = width.into();
        let height = height.into();

        Ok(Self {
            source_editor: SourceEditor::new(buffer, width, height - 1),
            file,
            stdout,
            width,
            height,
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

        let (lines, column, row) = self.source_editor.render();

        for line in lines {
            let line = if line.len() < self.width {
                line
            } else {
                &line[..self.width]
            };

            writeln!(self.stdout, "{}\r", line)?;
        }

        let file = match &self.file {
            Some(file) => file.display().to_string(),
            None => "[New File]".to_string(),
        };
        let status_bar = format!(" {file}{}", " ".repeat(self.width - file.len() - 1));
        write!(
            self.stdout,
            "{}",
            style::style(status_bar)
                .bold()
                .with(style::Color::DarkGrey)
                .on(style::Color::Black)
        )?;

        queue!(self.stdout, cursor::MoveTo(column as u16, row as u16))?;

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
                    event::KeyCode::Backspace => self.source_editor.backspace(),
                    event::KeyCode::Enter => self.source_editor.enter(),
                    event::KeyCode::Left => self.source_editor.left(),
                    event::KeyCode::Right => self.source_editor.right(),
                    event::KeyCode::Up => self.source_editor.up(),
                    event::KeyCode::Down => self.source_editor.down(),
                    event::KeyCode::Home => self.source_editor.home(),
                    event::KeyCode::End => self.source_editor.end(),
                    event::KeyCode::PageUp => todo!(),
                    event::KeyCode::PageDown => todo!(),
                    event::KeyCode::Tab => todo!(),
                    event::KeyCode::BackTab => todo!(),
                    event::KeyCode::Delete => todo!(),
                    event::KeyCode::Insert => todo!(),
                    event::KeyCode::F(_) => todo!(),
                    event::KeyCode::Char(c) => self.source_editor.keypress(c),
                    event::KeyCode::Null => {}
                    event::KeyCode::Esc => self.should_exit = true,
                },
                event::KeyEvent { .. } => {}
            },
            event::Event::Mouse(_) => {}
            event::Event::Resize(width, height) => {
                let width = width.into();
                let height = height.into();
                self.width = width;
                self.height = height;
                self.source_editor.resize(width, height - 1);
            }
        }

        std::net::TcpStream::connect("127.0.0.1:9292")
            .unwrap()
            .write_all(format!("\n\n\n\n\n\n\n\n{self:#?}").as_bytes())
            .unwrap();

        Ok(())
    }
}

#[derive(Debug)]
struct SourceEditor {
    buffer: Vec<String>,
    width: usize,
    height: usize,
    row: usize,
    column: usize,
    scroll: usize,
}

impl SourceEditor {
    fn new(buffer: String, width: usize, height: usize) -> Self {
        Self {
            buffer: buffer.split('\n').map(str::to_string).collect(),
            width,
            height,
            row: 0,
            column: 0,
            scroll: 0,
        }
    }

    fn render(&self) -> (Vec<&str>, usize, usize) {
        let mut lines = vec!["~"; self.height];

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

            lines[idx] = line;
        }

        (lines, self.column, self.row - self.scroll)
    }

    fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.scroll_to_show_cursor();
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
        self.scroll_to_show_cursor();
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
        self.scroll_to_show_cursor();
    }
    fn down(&mut self) {
        if self.row < self.buffer.len() - 1 {
            self.row += 1;
        }
        self.clamp_column();
        self.scroll_to_show_cursor();
    }
    fn home(&mut self) {
        self.column = 0;
    }
    fn end(&mut self) {
        self.column = self.buffer[self.row].len();
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

    fn clamp_column(&mut self) {
        let len = self.buffer[self.row].len();
        if self.column > len {
            self.column = len;
        }
    }
}
