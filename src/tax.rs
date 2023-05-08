use rust_decimal::Decimal;

pub struct Tax(Decimal);

impl Tax {
    pub fn new(value: Decimal) -> Tax {
        Tax { value }
    }

    pub fn get_value(&self) -> Decimal {
        self.0
    }
}
