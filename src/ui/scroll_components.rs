use tui::{
    style::{Modifier, Style},
    widgets::{Row, Table},
};

pub struct ScrollTable {
    content: Vec<Vec<String>>,
    focused: Option<usize>,
    selected: Option<usize>,
    displayable_results: usize,
}
impl ScrollTable {
    pub fn new(content: Vec<Vec<String>>) -> ScrollTable {
        ScrollTable {
            content,
            focused: None,
            selected: None,
            displayable_results: 0,
        }
    }
    pub fn focus(mut self, f: Option<usize>) -> ScrollTable {
        self.focused = f;
        self
    }
    pub fn selected(mut self, s: Option<usize>) -> ScrollTable {
        self.selected = s;
        self
    }
    pub fn displayable_results(mut self, r: usize) -> ScrollTable {
        self.displayable_results = r;
        self
    }
    pub fn render(self) -> Table<'static> {
        let mut items = self
            .content
            .into_iter()
            .map(|f| Row::new(f))
            .collect::<Vec<Row>>();
        if self.focused.is_some() && self.focused.unwrap() < items.len() {
            items[self.focused.unwrap()] = items
                .get(self.focused.unwrap())
                .unwrap()
                .to_owned()
                .style(Style::default().add_modifier(Modifier::BOLD));
        }
        if self.selected.is_some() && self.selected.unwrap() < items.len() {
            items[self.selected.unwrap()] = items
                .get(self.selected.unwrap())
                .unwrap()
                .to_owned()
                .style(Style::default().add_modifier(Modifier::ITALIC));
        }
        if self.selected.is_some()
            && self.focused.is_some()
            && self.focused.unwrap() == self.selected.unwrap()
            && self.focused.unwrap() < items.len()
        {
            items[self.selected.unwrap()] =
                items.get(self.selected.unwrap()).unwrap().to_owned().style(
                    Style::default()
                        .add_modifier(Modifier::ITALIC)
                        .add_modifier(Modifier::BOLD),
                );
        }
        if self.displayable_results > 0
            && self.focused.is_some()
            && self.focused.unwrap() > self.displayable_results
        {
            items.drain(0..self.focused.unwrap() - self.displayable_results);
        }
        Table::new(items)
    }
}
