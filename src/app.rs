use ratatui::widgets::ListState;

pub struct App {
    pub input: String,
    pub messages: Vec<String>,
    pub message_state: ListState,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> App {
        App {
            input: String::new(),
            messages: Vec::new(),
            message_state: ListState::default(),
            should_quit: false,
        }
    }

    pub fn handle_enter(&mut self) -> Option<String> {
        if !self.input.is_empty() {
            let message_to_send = self.input.clone();
            self.add_message(format!("Ben: {}", self.input));
            self.input.clear();
            Some(message_to_send)
        } else {
            None
        }
    }

    pub fn add_message(&mut self, message: String) {
        if !message.is_empty() {
            self.messages.push(message);
            if self.messages.len() > 50 {
                self.messages.remove(0);
            }
            self.message_state.select(Some(self.messages.len().saturating_sub(1)));
        }
    }

    pub fn scroll_up(&mut self) {
        let current_selection = self.message_state.selected().unwrap_or(0);
        if current_selection > 0 {
            self.message_state.select(Some(current_selection - 1));
        }
    }

    pub fn scroll_down(&mut self) {
        let current_selection = self.message_state.selected().unwrap_or(0);
        if !self.messages.is_empty() && current_selection < self.messages.len() - 1 {
            print!("aa");
            self.message_state.select(Some(current_selection + 1));
        } else if !self.messages.is_empty() {
            self.message_state.select(None);
            self.message_state.select(Some(current_selection - 1));
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        if !self.messages.is_empty() {
            self.message_state.select(Some(self.messages.len() - 1));
        } else {
            self.message_state.select(None);
        }
    }
}
