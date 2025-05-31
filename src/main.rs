use crossterm::terminal::{disable_raw_mode, enable_raw_mode, ClearType};
use crossterm::{execute, cursor::{MoveTo, MoveLeft, MoveRight}};
use crossterm::event::{read, Event, KeyCode, KeyEventKind};
use std::io::{stdout, Write};

fn main() {
    let mut stdout = stdout();

    enable_raw_mode().unwrap();

    execute!(
        stdout,
        crossterm::terminal::Clear(ClearType::All),
        MoveTo(0, 0)
    ).unwrap();

    let mut cursor_pos: usize = 0;
    let mut line = String::new();

    loop {
        if let Event::Key(event) = read().unwrap() {
            if event.kind == KeyEventKind::Press {
                match event.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(c) => {
                        line.insert(cursor_pos, c);
                        cursor_pos += 1;

                        write!(stdout, "{}", &line[cursor_pos-1..]).unwrap();

                        let len_after = line.len() - cursor_pos;
                        if len_after > 0 {
                            execute!(stdout, MoveLeft(len_after as u16)).unwrap();
                        }

                        stdout.flush().unwrap();
                    },
                    KeyCode::Enter => {
                        write!(stdout, "\r\n").unwrap();
                        stdout.flush().unwrap();
                        cursor_pos = 0;
                        line.clear();
                    },
                    KeyCode::Backspace => {
                        if cursor_pos > 0 {
                            cursor_pos -= 1;
                            line.remove(cursor_pos);

                            execute!(stdout, MoveLeft(1)).unwrap();

                            write!(stdout, "{} ", &line[cursor_pos..]).unwrap();

                            let len_after = line.len() - cursor_pos + 1;
                            execute!(stdout, MoveLeft(len_after as u16)).unwrap();

                            stdout.flush().unwrap();
                        }
                    },
                    KeyCode::Left => {
                        if cursor_pos > 0 {
                            execute!(stdout, MoveLeft(1)).unwrap();
                            cursor_pos -= 1;
                        }
                    },
                    KeyCode::Right => {
                        if cursor_pos < line.len() {
                            execute!(stdout, MoveRight(1)).unwrap();
                            cursor_pos += 1;
                        }
                    },
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode().unwrap();
}
