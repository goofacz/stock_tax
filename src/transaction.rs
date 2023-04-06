use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Display)]
pub enum Currency {
    PLN,
    USD,
    GBP,
    EUR,
}
