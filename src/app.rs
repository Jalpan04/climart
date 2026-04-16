use crate::models::tool::Tool;

#[derive(Debug, PartialEq)]
pub enum ExecutionState {
    Idle,
    Installing,
    Success(String),
    Error(String),
}

#[derive(Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
}

pub struct App {
    // Search
    pub query: String,
    pub input_mode: InputMode,

    // Results
    pub results: Vec<Tool>,
    pub selected_index: usize,
    pub detail_scroll: u16,

    // Status
    pub is_loading: bool,
    pub error: Option<String>,
    pub status_message: Option<String>,

    // Execution
    pub execution_state: ExecutionState,

    // Search history (last 10 queries)
    pub search_history: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        App {
            query: String::new(),
            input_mode: InputMode::Normal,
            results: Vec::new(),
            selected_index: 0,
            detail_scroll: 0,
            is_loading: false,
            error: None,
            status_message: None,
            execution_state: ExecutionState::Idle,
            search_history: Vec::new(),
        }
    }

    pub fn search_focused(&self) -> bool {
        self.input_mode == InputMode::Search
    }

    pub fn enter_search(&mut self) {
        self.input_mode = InputMode::Search;
    }

    pub fn leave_search(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    pub fn select_next(&mut self) {
        if !self.results.is_empty() && self.selected_index < self.results.len() - 1 {
            self.selected_index += 1;
            self.detail_scroll = 0;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.detail_scroll = 0;
        }
    }

    pub fn scroll_detail_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(1);
    }

    pub fn scroll_detail_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
    }

    pub fn set_loading(&mut self) {
        self.is_loading = true;
        self.error = None;
        self.status_message = Some(format!("Searching for '{}'...", self.query));
    }

    pub fn set_results(&mut self, results: Vec<Tool>) {
        let count = results.len();
        self.results = results;
        self.selected_index = 0;
        self.detail_scroll = 0;
        self.is_loading = false;
        self.error = None;
        if count == 0 {
            self.status_message = Some("No results found.".to_string());
        } else {
            self.status_message = Some(format!("Found {} results.", count));
        }
    }

    pub fn set_error(&mut self, msg: String) {
        self.error = Some(msg.clone());
        self.is_loading = false;
        self.status_message = None;
    }

    pub fn add_char(&mut self, c: char) {
        self.query.push(c);
    }

    pub fn remove_char(&mut self) {
        self.query.pop();
    }

    #[allow(dead_code)]
    pub fn clear_query(&mut self) {
        self.query.clear();
    }

    pub fn push_history(&mut self, query: &str) {
        if !query.is_empty() {
            self.search_history.retain(|q| q != query);
            self.search_history.insert(0, query.to_string());
            self.search_history.truncate(10);
        }
    }

    pub fn current_tool(&self) -> Option<&Tool> {
        self.results.get(self.selected_index)
    }

    pub fn has_results(&self) -> bool {
        !self.results.is_empty()
    }

    pub fn tick(&mut self) {
        // Called on every UI tick — used to drive spinner animation in the future
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::tool::Tool;

    fn dummy_tool(name: &str) -> Tool {
        Tool {
            name: name.to_string(),
            description: "Dummy tool".to_string(),
            source: "npm".to_string(),
            version: "1.0.0".to_string(),
            binary: Some(name.to_string()),
            install_command: format!("npm install -g {}", name),
            run_command: format!("npx {}", name),
        }
    }

    #[test]
    fn test_app_initialization() {
        let app = App::new();
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.query, "");
        assert_eq!(app.results.len(), 0);
        assert!(!app.search_focused());
    }

    #[test]
    fn test_search_mode_transitions() {
        let mut app = App::new();
        app.enter_search();
        assert_eq!(app.input_mode, InputMode::Search);
        assert!(app.search_focused());

        app.leave_search();
        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(!app.search_focused());
    }

    #[test]
    fn test_query_modification() {
        let mut app = App::new();
        app.add_char('c');
        app.add_char('l');
        app.add_char('i');
        assert_eq!(app.query, "cli");

        app.remove_char();
        assert_eq!(app.query, "cl");

        app.clear_query();
        assert_eq!(app.query, "");
    }

    #[test]
    fn test_list_navigation() {
        let mut app = App::new();
        let tools = vec![
            dummy_tool("tool1"),
            dummy_tool("tool2"),
            dummy_tool("tool3"),
        ];
        app.set_results(tools);

        assert_eq!(app.selected_index, 0);

        app.select_next();
        assert_eq!(app.selected_index, 1);

        app.select_next();
        assert_eq!(app.selected_index, 2);

        app.select_next();
        assert_eq!(app.selected_index, 2);

        app.select_prev();
        assert_eq!(app.selected_index, 1);

        app.select_prev();
        app.select_prev();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_search_history() {
        let mut app = App::new();
        app.push_history("rust");
        app.push_history("python");
        app.push_history("rust"); // Should deduplicate and move to top

        assert_eq!(app.search_history, vec!["rust", "python"]);
    }
}
