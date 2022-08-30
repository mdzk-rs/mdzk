use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Arc {
    Connected(Vec<Range<usize>>),
    NotConnected,
}

impl Arc {
    pub fn push_link_range(&mut self, range: Range<usize>) {
        match self {
            Arc::Connected(ranges) => ranges.push(range),
            Arc::NotConnected => *self = Arc::Connected(vec![range]),
        }
    }
}
