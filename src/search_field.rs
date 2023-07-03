pub struct SearchField {
    text: String,
}

impl SearchField {
    pub fn new() -> Self {
        Self {
            text: String::new()
        }
    }

    pub fn get_display(&self, width: usize) -> String {
        let formatted = format!("Search: {}", self.text);
        
        if formatted.len() <= width {
            format!("{:<width$}", formatted, width = width)
        } else {
            formatted.get(..width).unwrap().to_owned()
        }
    }

    pub fn push_char(&mut self, c: char) {
        self.text.push(c);
    }

    pub fn delete_last(&mut self) {
        self.text.pop();
    }

    pub fn clear(&mut self) {
        self.text = String::new();
    }

    pub fn get_text(&self) -> String {
        self.text.clone()
    } 
}

#[cfg(test)]
mod tests {
    use super::SearchField;

    #[test]
    fn fitting_length() {
        let mut text = SearchField::new();
        text.push_char('C');
        text.push_char('C');

        assert_eq!(text.get_display(10), "Search: CC");
    }

    #[test]
    fn too_small_to_fit() {
        let mut text = SearchField::new();
        text.push_char('C');
        text.push_char('C');

        assert_eq!(text.get_display(4), "Sear");
    }
    
    #[test]
    fn longer_than_text() {
        let mut text = SearchField::new();
        text.push_char('C');
        text.push_char('C');

        assert_eq!(text.get_display(12), "Search: CC  ");
    }

    #[test]
    fn after_deletion() {
        let mut text = SearchField::new();
        text.push_char('C');
        assert_eq!(text.get_display(10), "Search: C ");
        text.delete_last();
        assert_eq!(text.get_display(10), "Search:   ");
    }



}