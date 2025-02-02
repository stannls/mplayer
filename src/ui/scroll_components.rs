use ratatui::{
    layout::Constraint, style::{Modifier, Style}, widgets::{Row, Table}
};

pub struct ScrollTable {
    content: Vec<Vec<String>>,
    constraints: Vec<Constraint>,
    content_range: (usize, usize),
    focused: Option<usize>,
    selected: Option<usize>,
    displayable_results: usize,
}
impl ScrollTable {
    pub fn new(content: Vec<Vec<String>>, constraints: Vec<Constraint>) -> ScrollTable {
        ScrollTable {
            content,
            constraints,
            content_range: (0, 0),
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
        if self.content_range.1 - self.content_range.0 > r {
            self.content_range.1 = self.content_range.0 + r;
        }
        self
    }
    pub fn render(&mut self) -> Table<'static> {
        let mut items = self
            .content
            .clone()
            .into_iter()
            .map(Row::new)
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
        // Check if the focused item is not fitting into the default display
        if self.focused.is_some() && (self.focused.unwrap() < self.content_range.0 || self.focused.unwrap() > self.content_range.1) {
            if self.focused.unwrap() < self.content_range.0 {
                self.content_range = (self.focused.unwrap(), self.focused.unwrap() + self.displayable_results);
            } else if self.focused.unwrap() > self.content_range.1 {
                self.content_range = (self.focused.unwrap() - self.displayable_results, self.focused.unwrap());
            }
        }
        items.drain(0..self.content_range.0);
        Table::new(items, &self.constraints)
    }
}
