use crossterm::{execute, cursor::{MoveTo, MoveLeft, MoveRight}};
use crossterm::event::{read, Event, KeyCode, KeyEventKind};
use std::io::{stdout, Write, Stdout};
use crossterm::cursor::{MoveDown, MoveUp};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType};

pub struct Editor {
    cursor_pos: usize,
    line_pos: usize,
    terminal_height: u16,
    terminal_width: u16,
    scroll_offset: usize,
    lines: Vec<String>,
    stdout: Stdout,
    insert_mode: bool,
}

impl Editor {
    pub fn new() -> Editor {
        let (cols, rows) = crossterm::terminal::size().unwrap();
        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        execute!(
            stdout,
            Clear(ClearType::All),
            Clear(ClearType::Purge),
            MoveTo(0, 0)
        ).unwrap();

        Editor {
            cursor_pos: 0,
            line_pos: 0,
            lines: vec![String::new()],
            stdout,
            terminal_height: rows,
            terminal_width: cols,
            scroll_offset: 0,
            insert_mode: false,
        }
    }

    fn redraw_screen(&mut self) {
        // More aggressive clearing
        execute!(
        self.stdout,
        Clear(ClearType::All),
        Clear(ClearType::Purge),
        MoveTo(0, 0)
    ).unwrap();

        let content_height = self.terminal_height.saturating_sub(1) as usize;
        let visible_lines = content_height.min(self.lines.len() - self.scroll_offset);

        // Draw visible lines
        for i in 0..visible_lines {
            let line_idx = i + self.scroll_offset;
            let line = &self.lines[line_idx];
            let line_num = line_idx + 1;

            execute!(self.stdout, MoveTo(0, i as u16)).unwrap();
            // Clear the line completely first
            execute!(self.stdout, Clear(ClearType::CurrentLine)).unwrap();
            write!(self.stdout, "{} {}", line_num, line).unwrap();
        }

        // Explicitly clear any remaining lines that might have old content
        for i in visible_lines..content_height {
            execute!(self.stdout, MoveTo(0, i as u16)).unwrap();
            execute!(self.stdout, Clear(ClearType::CurrentLine)).unwrap();
        }

        // Draw status bar
        let status_y = self.terminal_height - 1;
        execute!(self.stdout, MoveTo(0, status_y)).unwrap();
        execute!(self.stdout, Clear(ClearType::CurrentLine)).unwrap();
        let mode_text = if self.insert_mode { "INSERT" } else { "COMMAND" };
        write!(self.stdout, "-- {} -- Line: {}, Col: {}",
               mode_text, self.line_pos + 1, self.cursor_pos + 1).unwrap();

        // Position cursor
        let cursor_y = (self.line_pos - self.scroll_offset) as u16;
        let line_num_width = (self.line_pos + 1).to_string().len() as u16;
        let cursor_x = line_num_width + 1 + self.cursor_pos as u16;

        execute!(self.stdout, MoveTo(cursor_x, cursor_y)).unwrap();
        self.stdout.flush().unwrap();
    }

    fn clear_screen_manual(&mut self) {
        for row in 0..self.terminal_height {
            execute!(self.stdout, MoveTo(0, row)).unwrap();
            // Write spaces to fill the entire row
            write!(self.stdout, "{}", " ".repeat(self.terminal_width as usize)).unwrap();
        }
        execute!(self.stdout, MoveTo(0, 0)).unwrap();
    }

    fn redraw_screen_manual(&mut self) {
        self.clear_screen_manual();

        let content_height = self.terminal_height.saturating_sub(1) as usize;
        let visible_lines = content_height.min(self.lines.len() - self.scroll_offset);

        for i in 0..visible_lines {
            let line_idx = i + self.scroll_offset;
            let line = &self.lines[line_idx];
            let line_num = line_idx + 1;

            execute!(self.stdout, MoveTo(0, i as u16)).unwrap();
            write!(self.stdout, "{} {}", line_num, line).unwrap();
        }

        let status_y = self.terminal_height - 1;
        execute!(self.stdout, MoveTo(0, status_y)).unwrap();
        let mode_text = if self.insert_mode { "INSERT" } else { "COMMAND" };
        write!(self.stdout, "-- {} -- Line: {}, Col: {}",
               mode_text, self.line_pos + 1, self.cursor_pos + 1).unwrap();

        let cursor_y = (self.line_pos - self.scroll_offset) as u16;
        let line_num_width = (self.line_pos + 1).to_string().len() as u16;
        let cursor_x = line_num_width + 1 + self.cursor_pos as u16;

        execute!(self.stdout, MoveTo(cursor_x, cursor_y)).unwrap();
        self.stdout.flush().unwrap();
    }

    fn adjust_scroll(&mut self) {
        let content_height = self.terminal_height.saturating_sub(1) as usize;

        if self.line_pos >= self.scroll_offset + content_height {
            self.scroll_offset = self.line_pos - content_height + 1;
        }
        else if self.line_pos < self.scroll_offset {
            self.scroll_offset = self.line_pos;
        }
    }

    pub fn run(&mut self) {
        self.redraw_screen();

        loop {
            if let Event::Key(event) = read().unwrap() {
                if event.kind == KeyEventKind::Press {
                    match event.code {
                        KeyCode::Char('q') => {
                            if !self.insert_mode {
                                break;
                            } else {
                                self.lines[self.line_pos].insert(self.cursor_pos, 'q');
                                self.cursor_pos += 1;
                                self.redraw_screen();
                            }
                        },
                        KeyCode::Esc => {
                            if self.insert_mode {
                                self.insert_mode = false;
                                self.redraw_screen();
                            } else {
                                break;
                            }
                        },
                        KeyCode::Char(c) => {
                            if c == 'i' && !self.insert_mode {
                                self.insert_mode = true;
                                self.redraw_screen();
                            } else if self.insert_mode {
                                self.lines[self.line_pos].insert(self.cursor_pos, c);
                                self.cursor_pos += 1;
                                self.redraw_screen();
                            }
                        },
                        KeyCode::Enter => {
                            if self.insert_mode {
                                let new_line = self.lines[self.line_pos].split_off(self.cursor_pos);
                                self.line_pos += 1;
                                self.cursor_pos = 0;
                                self.lines.insert(self.line_pos, new_line);

                                self.adjust_scroll();
                                self.redraw_screen();
                            }
                        }
                        KeyCode::Backspace => {
                            if self.insert_mode {
                                if self.cursor_pos > 0 {
                                    self.cursor_pos -= 1;
                                    self.lines[self.line_pos].remove(self.cursor_pos);
                                    self.redraw_screen();
                                } else if self.line_pos > 0 {
                                    let current_line = self.lines.remove(self.line_pos);
                                    self.line_pos -= 1;
                                    self.cursor_pos = self.lines[self.line_pos].len();
                                    self.lines[self.line_pos].push_str(&current_line);

                                    self.adjust_scroll();
                                    self.redraw_screen();
                                }
                            }
                        },
                        KeyCode::Left => {
                            if self.cursor_pos > 0 {
                                self.cursor_pos -= 1;
                                self.redraw_screen();
                            } else if self.line_pos > 0 {
                                self.line_pos -= 1;
                                self.cursor_pos = self.lines[self.line_pos].len();
                                self.adjust_scroll();
                                self.redraw_screen();
                            }
                        },
                        KeyCode::Right => {
                            if self.cursor_pos < self.lines[self.line_pos].len() {
                                self.cursor_pos += 1;
                                self.redraw_screen();
                            } else if self.line_pos + 1 < self.lines.len() {
                                self.line_pos += 1;
                                self.cursor_pos = 0;
                                self.adjust_scroll();
                                self.redraw_screen();
                            }
                        },
                        KeyCode::Up => {
                            if self.line_pos > 0 {
                                self.line_pos -= 1;
                                self.cursor_pos = self.cursor_pos.min(self.lines[self.line_pos].len());
                                self.adjust_scroll();
                                self.redraw_screen();
                            }
                        },
                        KeyCode::Down => {
                            if self.line_pos + 1 < self.lines.len() {
                                self.line_pos += 1;
                                self.cursor_pos = self.cursor_pos.min(self.lines[self.line_pos].len());
                                self.adjust_scroll();
                                self.redraw_screen();
                            }
                        },
                        _ => {}
                    }
                }
            }
        }

        execute!(
            self.stdout,
            Clear(ClearType::All),
            MoveTo(0, 0)
        ).unwrap();
        disable_raw_mode().unwrap();
    }
}