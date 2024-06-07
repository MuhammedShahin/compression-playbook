use std::cmp::Ordering;

#[derive(Eq, PartialEq)]
pub struct Node {
    pub(crate) character: Option<char>,
    pub(crate) freq: u32,
    pub(crate) left_node: Option<Box<Node>>,
    pub(crate) right_node: Option<Box<Node>>,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.freq.cmp(&self.freq)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
