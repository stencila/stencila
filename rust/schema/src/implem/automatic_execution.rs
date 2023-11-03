use crate::AutomaticExecution;

impl AutomaticExecution {
    pub fn to_html_special(&self) -> String {
        self.to_string().to_lowercase()
    }
}
