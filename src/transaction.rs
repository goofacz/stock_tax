use std::fmt;

#[derive(Debug)]
pub enum Currency {
    PLN,
    USD,
    GBP,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
