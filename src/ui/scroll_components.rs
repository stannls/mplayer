use tui::{
    style::{Modifier, Style},
    widgets::{Row, Table},
};

pub struct ScrollTable {
    content: Vec<Vec<String>>,
    focused: Option<usize>,
    displayable_results: usize,
}
impl ScrollTable {
    pub fn new(content: Vec<Vec<String>>) -> ScrollTable {
        ScrollTable {
            content,
            focused: None,
            displayable_results: 0,
        }
    }
    pub fn focus(mut self, f: Option<usize>) -> ScrollTable {
        self.focused = f;
        self
    }
    pub fn displayable_results(mut self, r: usize) -> ScrollTable {
        self.displayable_results = r;
        self
    }
    pub fn render(self) -> Table<'static> {
        let items = match self.focused {
            Some(focus) => {
                let mut result = vec![];
                for i in 0..self.content.len() {
                    if i != focus as usize {
                        result.push(Row::new(self.content.get(i).unwrap().clone()));
                    } else {
                        result.push(
                            Row::new(self.content.get(i).unwrap().clone())
                                .style(Style::default().add_modifier(Modifier::BOLD)),
                        );
                    }
                }
                if self.displayable_results > 0 && focus > self.displayable_results {
                    result.drain(0..focus - self.displayable_results);
                }
                result
            }
            None => self.content.into_iter().map(|f| Row::new(f)).collect(),
        };
        Table::new(items)
    }
}
