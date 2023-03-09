//! Utilities for filtering output from tex engines, etc.

struct UndefinedControlSequence {
    file: String,
    linum: usize,
    /// The source line containing the error.
    src: String,
}

pub trait InfoItem {}

impl InfoItem for UndefinedControlSequence {}

pub struct Info {
    items: Vec<Box<dyn InfoItem>>,
}

impl Info {
    fn push<I: InfoItem>(&mut self, item: I) {
        self.items.push(Box::new(item));
    }
}

trait FilterParser {}

struct Filter {
}

pub fn filter_errors<R: std::io::BufRead>(output: R) -> crate::Result<Info> {
    let mut lines = output.lines();
    for line in lines.into_iter() {
        self.
    }
    Ok(())
}
