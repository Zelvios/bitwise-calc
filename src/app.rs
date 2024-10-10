use crate::input_mode::InputMode;
use color_eyre::Result;
use ratatui::prelude::{Style, Stylize};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style as RatatuiStyle},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph},
    DefaultTerminal, Frame as RatatuiFrame,
};
use tui_big_text::{BigText, PixelSize};

pub struct App {
    input: String,
    first_number: Option<i32>,
    second_number: Option<i32>,
    operator: Option<String>,
    character_index: usize,
    input_mode: InputMode,
    messages: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    #[allow(clippy::must_use_candidate)]
    pub const fn new() -> Self {
        Self {
            input: String::new(),
            first_number: None,
            second_number: None,
            operator: None,
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            character_index: 0,
        }
    }

    #[allow(clippy::must_use_candidate)]
    pub const fn add(first: i32, second: i32) -> i32 {
        first + second
    }
    #[allow(clippy::must_use_candidate)]
    pub const fn subtract(first: i32, second: i32) -> i32 {
        first - second
    }
    #[allow(clippy::must_use_candidate)]
    pub const fn div(first: i32, second: i32) -> i32 {
        first / second
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        if self.character_index != 0 {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_message(&mut self) {
        if self.first_number.is_none() {
            if let Ok(num) = self.input.trim().parse::<i32>() {
                self.first_number = Some(num);
                self.messages.push(format!("First number entered: {num}"));
            } else {
                self.messages
                    .push("Invalid input! Please enter a valid first number.".to_string());
            }
        } else if self.second_number.is_none() {
            if let Ok(num) = self.input.trim().parse::<i32>() {
                self.second_number = Some(num);
                self.messages.push(format!("Second number entered: {num}"));
            } else {
                self.messages
                    .push("Invalid input! Please enter a valid second number.".to_string());
            }
        } else if self.operator.is_none() {
            let trimmed_input = self.input.trim().to_lowercase();
            match trimmed_input.as_str() {
                "+" | "plus" => self.operator = Some("+".to_string()),
                "-" | "minus" => self.operator = Some("-".to_string()),
                "/" | "div" => self.operator = Some("/".to_string()),
                _ => {
                    self.messages.push("Invalid operator! Please enter a valid operator: '+' (plus), '-' (minus), or '/' (div).".to_string());
                }
            }
        }

        if let (Some(first), Some(second), Some(operator)) =
            (self.first_number, self.second_number, &self.operator)
        {
            let result = match operator.as_str() {
                "+" => Self::add(first, second),
                "-" => Self::subtract(first, second),
                "/" => Self::div(first, second),
                _ => unreachable!(),
            };

            self.messages
                .push(format!("{first} {operator} {second} = {result}"));

            self.first_number = None;
            self.second_number = None;
            self.operator = None;
        }

        self.input.clear();
        self.reset_cursor();
    }

    fn clear_messages(&mut self) {
        self.messages.clear();
        self.first_number = None;
        self.second_number = None;
        self.operator = None;
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            self.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Char('c') => {
                            self.clear_messages();
                        }
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => self.submit_message(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Esc => self.input_mode = InputMode::Normal,
                        _ => {}
                    },
                    InputMode::Editing => {}
                }
            }
        }
    }

    fn draw(&self, frame: &mut RatatuiFrame) {
        // Create the big text
        let big_text = BigText::builder()
            .pixel_size(PixelSize::HalfHeight)
            .style(Style::new().blue())
            .lines(vec!["Bitwise-Calc".into()])
            .build();

        let vertical = Layout::vertical([
            Constraint::Length(1), // Space for top
            Constraint::Length(5), // Big text / title
            Constraint::Length(4), // Info box area
            Constraint::Length(3), // Input area
            Constraint::Min(1),    // Messages area
        ]);

        // Get the areas based on the vertical layout
        let [_top_space, big_text_area, help_area, input_area, messages_area] =
            vertical.areas(frame.area());

        frame.render_widget(big_text, big_text_area);

        // Determine the info message based on input mode
        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    Line::from(vec![
                        Span::styled("'q'", Style::default().fg(Color::Red)),
                        Span::raw(" to exit"),
                    ]),
                    Line::from(vec![
                        Span::styled("'e'", Style::default().fg(Color::Yellow)),
                        Span::raw(" to start editing"),
                    ]),
                    Line::from(vec![
                        Span::styled("'c'", Style::default().fg(Color::LightBlue)),
                        Span::raw(" to clear messages"),
                    ]),
                ],
                RatatuiStyle::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => {
                let prompt = if self.first_number.is_none() {
                    vec![Line::from(Span::styled(
                        "Please enter the first number",
                        Style::default().fg(Color::Yellow),
                    ))]
                } else if self.second_number.is_none() {
                    vec![Line::from(Span::styled(
                        "Please enter the second number",
                        Style::default().fg(Color::Yellow),
                    ))]
                } else {
                    vec![
                        Line::from(Span::styled(
                            "Please enter your operation",
                            Style::default().fg(Color::Yellow),
                        )),
                        Line::from(Span::styled(
                            "➣ '+' or 'plus'",
                            Style::default().fg(Color::Green),
                        )),
                        Line::from(Span::styled(
                            "➣ '-' or 'minus'",
                            Style::default().fg(Color::Green),
                        )),
                    ]
                };

                (prompt, RatatuiStyle::default())
            }
        };

        // Info message area
        let text = Text::from(msg);
        let help_message = Paragraph::new(text).style(style);
        frame.render_widget(help_message, help_area);

        // Input area
        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => RatatuiStyle::default(),
                InputMode::Editing => RatatuiStyle::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);

        // Set the cursor position if in editing mode
        #[allow(clippy::cast_possible_truncation)]
        if matches!(self.input_mode, InputMode::Editing) {
            frame.set_cursor_position(Position::new(
                input_area.x + self.character_index as u16 + 1,
                input_area.y + 1,
            ));
        }

        // Messages area
        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .enumerate()
            .rev()
            .map(|(i, m)| {
                let content = if m.contains('=') {
                    Span::styled(format!("{i}: {m}"), Style::default().fg(Color::Green))
                } else if m.contains("Invalid") {
                    Span::styled(format!("{i}: {m}"), Style::default().fg(Color::Red))
                } else {
                    Span::styled(format!("{i}: {m}"), Style::default().fg(Color::White))
                };
                ListItem::new(Line::from(content))
            })
            .collect();
        let messages = List::new(messages).block(Block::bordered().title("Messages"));
        frame.render_widget(messages, messages_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_reset_cursor() {
        let mut app = App::new();
        app.character_index = 3;
        app.reset_cursor();
        assert_eq!(app.character_index, 0);
    }
    #[test]
    fn test_add() {
        assert_eq!(App::add(2, 3), 5);
        assert_eq!(App::add(-1, 1), 0);
    }
    #[test]
    fn test_subtract() {
        assert_eq!(App::subtract(5, 3), 2);
        assert_eq!(App::subtract(0, 1), -1);
    }
    #[test]
    fn test_divide() {
        assert_eq!(App::div(6, 3), 2);
    }

    #[test]
    fn test_clear_messages() {
        let mut app = App::new();

        app.first_number = Some(42);
        app.second_number = Some(7);
        app.operator = Some("+".to_string());
        app.messages.push("First number entered: 42".to_string());
        app.messages.push("Second number entered: 7".to_string());

        app.clear_messages();

        assert!(app.first_number.is_none());
        assert!(app.second_number.is_none());
        assert!(app.operator.is_none());
        assert!(app.messages.is_empty());
    }
}
