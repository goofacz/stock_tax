use crate::activity::{Activity, Money, Operation};
use crate::currency::Pln;
use chrono::Datelike;
use chrono::NaiveDateTime;
use clap::Args;
use colored::Colorize;
use derive_more::{AddAssign, Display, Error};
use glob::glob;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json;
use std::cmp::min;
use std::collections::{HashMap, HashSet, VecDeque};
use std::error;
use std::fs::OpenOptions;
use std::io::BufReader;

#[derive(Display, Error, Debug)]
pub struct Error {
    reason: String,
}

#[derive(Args)]
pub struct CommandArgs {
    path: String,
}

#[derive(Debug)]
struct Block {
    timestamp: NaiveDateTime,
    quantity: Decimal,
    price: Pln,
    commission: Pln,
}

#[derive(Debug, Default)]
struct Stock {
    blocks: HashMap<String, VecDeque<Block>>,
}

#[derive(Default)]
struct TaxPosition {
    dividend: Pln,
    dividend_withholding_tax: Pln,
    stock_revenue: Pln,
    stock_cost: Pln,
}

#[derive(Default, AddAssign)]
struct TaxReturn {
    dividend_tax: Pln,
    stock_revenue: Pln,
    stock_cost: Pln,
    stock_income: Pln,
    stock_loss: Pln,
    stock_tax: Pln,
}

impl Block {
    fn new(
        timestamp: &NaiveDateTime,
        quantity: &Decimal,
        price: &Money,
        commission: &Money,
    ) -> Block {
        Block {
            timestamp: *timestamp,
            quantity: *quantity,
            price: price.pln,
            commission: commission.pln,
        }
    }
}

impl Stock {
    fn get_blocks(&mut self, symbol: String) -> &mut VecDeque<Block> {
        self.blocks.entry(symbol).or_default()
    }

    fn buy(&mut self, activity: &Activity, quantity: &Decimal, price: &Money, commission: &Money) {
        println!("{date}: {symbol}: {prefix} quantity: {quantity} price: {price_pln} ({price_org}) commission: {commission_pln} ({commission_org})",
            date=activity.timestamp.date(),
            symbol=activity.symbol,
            prefix="Buy".green(),
            quantity=quantity,
            price_pln=price.pln,
            price_org=price.original,
            commission_pln=commission.pln,
            commission_org=commission.original);

        let blocks = self.get_blocks(activity.symbol.to_string());
        let block = Block::new(&activity.timestamp, quantity, price, commission);
        blocks.push_back(block);
    }

    fn sell(
        &mut self,
        activity: &Activity,
        quantity: &Decimal,
        price: &Money,
        commission: &Money,
    ) -> (Pln, Pln) {
        let blocks = self.get_blocks(activity.symbol.to_string());
        let mut sell_quantity = *quantity;
        let revenue = Pln((price.pln.0 * sell_quantity).round_dp(2));
        let sell_commission = commission.pln;
        let mut cost = Pln::default();
        let mut buy_commission = Pln::default();
        let mut logs = vec![];

        while sell_quantity > dec!(0) {
            let mut block = blocks.pop_front().unwrap();

            let quantity = min(block.quantity, sell_quantity);
            block.quantity -= quantity;
            sell_quantity -= quantity;

            let block_cost = Pln((block.price.0 * quantity).round_dp(2));
            let block_buy_commission = if block.quantity == dec!(0) {
                block.commission
            } else {
                Pln::default()
            };

            let log = format!("  {date}: Sell block quantity: {quantity} cost: {cost} price: {price} commission: {commission}",
                date=block.timestamp.date(),
                cost=block_cost,
                price=block.price,
                commission=block_buy_commission,
            );
            logs.push(log);

            cost += block_cost;
            buy_commission += block_buy_commission;

            if block.quantity > dec!(0) {
                blocks.push_front(block);
            }
        }

        let cost = cost + buy_commission + sell_commission;
        let value = revenue - cost;
        let (income, loss) = if value.0 > dec!(0) {
            (value, Pln::default())
        } else {
            (Pln::default(), Pln(value.0.abs()))
        };

        println!("{date}: {symbol}: {prefix} quantity: {quantity} cost: {cost} revenue: {revenue} income: {income} loss: {loss} price: {price_pln} ({price_org}) commission: {commission_pln} ({commission_org})",
            date=activity.timestamp.date(),
            symbol=activity.symbol,
            prefix="Sell".red(),
            price_pln=price.pln,
            price_org=price.original,
            commission_pln=commission.pln,
            commission_org=commission.original);

        for log in logs {
            println!("{}", log);
        }

        (revenue, cost)
    }
}

fn load_activities(path: &String) -> Result<Vec<Activity>, Box<dyn error::Error>> {
    let mut activities = vec![];
    for file_path in glob(path)? {
        let file_path = file_path?;
        let file = OpenOptions::new().read(true).open(file_path)?;
        let reader = BufReader::new(file);
        let file_activities: Vec<Activity> = serde_json::from_reader(reader)?;
        activities.extend(file_activities.into_iter());
    }

    activities.sort_by_key(|activity| activity.timestamp);
    Ok(activities)
}

fn process_dividend(activity: &Activity, value: &Money, withholding_tax: &Money) -> (Pln, Pln) {
    println!(
        "{date}: {symbol}: {prefix} value: {value} / {value_pln}: tax: {tax} / {tax_pln}",
        date = activity.timestamp.date(),
        symbol = activity.symbol,
        prefix = "Dividend".yellow(),
        value = value.original,
        value_pln = value.pln,
        tax = withholding_tax.original,
        tax_pln = withholding_tax.pln,
    );
    (value.pln, withholding_tax.pln)
}

fn process_annual_activities<'a>(
    stock: &mut Stock,
    year: i32,
    activities: impl Iterator<Item = &'a Activity>,
) {
    let mut tax_positions = HashMap::<&str, TaxPosition>::new();

    for activity in activities {
        match &activity.operation {
            Operation::Dividend {
                value,
                withholding_tax,
            } => {
                let (dividend, withholding_tax) =
                    process_dividend(activity, value, withholding_tax);
                let tax_position = tax_positions.entry(&activity.symbol).or_default();
                tax_position.dividend += dividend;
                tax_position.dividend_withholding_tax += withholding_tax;
            }
            Operation::Buy {
                quantity,
                price,
                commission,
            } => {
                stock.buy(activity, quantity, price, commission);
            }
            Operation::Sell {
                quantity,
                price,
                commission,
            } => {
                let (revenue, cost) = stock.sell(activity, quantity, price, commission);
                let tax_position = tax_positions.entry(&activity.symbol).or_default();
                tax_position.stock_revenue += revenue;
                tax_position.stock_cost += cost;
            }
        }
    }

    let tax_return = tax_positions.iter().map(|(symbol, tax_position)|{
        let dividend_tax = Pln((tax_position.dividend.0 * dec!(0.19)).round_dp(2)) - tax_position.dividend_withholding_tax;
        let value = tax_position.stock_revenue - tax_position.stock_cost;
        let (stock_income, stock_loss) = if value.0 > dec!(0) {
            (value, Pln::default())
        } else {
            (Pln::default(), Pln(value.0.abs()))
        };
        let stock_tax = Pln((stock_income.0 * dec!(0.19)).round_dp(2));

        println!("{year}: {symbol} dividend tax: {dividend_tax} stock income: {stock_income} stock loss: {stock_loss} stock_tax: {stock_tax}");

        TaxReturn {
            dividend_tax: dividend_tax,
            stock_revenue: tax_position.stock_revenue,
            stock_cost: tax_position.stock_cost,
            stock_income: stock_income,
            stock_loss: stock_loss,
            stock_tax: stock_tax,
        }
    }).fold(TaxReturn::default(), |mut acc, a| { acc +=a; acc });

    let summary = format!("{year}: {prefix} dividend tax: {dividend_tax} stock revenue: {stock_revenue} stock cost: {stock_cost} stock income: {stock_income} stock loss: {stock_loss} stock tax: {stock_tax}",
        prefix="TAX RETURN".bright_blue(),
        dividend_tax=Pln((tax_return.dividend_tax.0 * dec!(0.19)).round_dp(0)),
        stock_revenue=tax_return.stock_revenue,
        stock_cost=tax_return.stock_cost,
        stock_income=tax_return.stock_income,
        stock_loss=tax_return.stock_loss,
        stock_tax=tax_return.stock_tax,
    );
    println!("{}", summary.bold());
}

pub fn command(args: &CommandArgs) -> Result<(), Box<dyn error::Error>> {
    let mut stock = Stock::default();
    let activities = load_activities(&args.path)?;

    let years = activities.iter().map(|a| a.timestamp.year());
    let mut years: Vec<_> = HashSet::<i32>::from_iter(years).into_iter().collect();
    years.sort();

    for year in years {
        let activities = activities
            .iter()
            .filter(|a| a.timestamp.year() == year)
            .into_iter();
        process_annual_activities(&mut stock, year, activities);
    }

    Ok(())
}
