use crate::label::Label;

#[derive(PartialEq, Debug)]
pub struct Target {
    pub inputs: Vec<Label>,
    pub outputs: Vec<Label>,
}

impl Target {
    pub fn new(inputs: Vec<Label>, outputs: Vec<Label>) -> Self {
        Self { inputs, outputs }
    }
}
