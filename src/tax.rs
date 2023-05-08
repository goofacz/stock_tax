use rust_decimal::Decimal;

pub struct Tax(Decimal);

impl Tax {
    pub fn new(value: i64) -> Tax {
        Tax(Decimal::new(value, 2))
    }

    pub fn get_value(&self) -> Decimal {
        self.0
    }
}
