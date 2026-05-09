use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ShikiApiFranchise {
    pub links: Vec<Link>,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Link {
    pub source_id: usize,
    pub target_id: usize,
    pub relation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Node {
    pub id: usize,
    pub name: String,
}
