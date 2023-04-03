use csv::ReaderBuilder;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Transaction {
    #[serde(rename(deserialize = "Symbol"))]
    symbol: String,
    #[serde(rename(deserialize = "Quantity"))]
    quantity: i32,
}

pub fn convert(path: &Path) -> Result<String, Box<dyn Error>> {
    let handle = File::open(path)?;
    let reader = BufReader::new(handle);
    let lines: Vec<_> = reader.lines().collect::<Result<Vec<_>, _>>()?;

    let transactions = lines
        .iter()
        .filter(|line| {
            let header = "Trades,Header,DataDiscriminator,Asset Category,Currency,Symbol,Date/Time,Quantity,T. Price,C. Price,Proceeds,Comm/Fee,Basis,Realized P/L,MTM P/L,Code";
            let prefix = "Trades,Data,Order,Stocks";
            line.starts_with(header) || line.starts_with(prefix)
        })
        .collect::<Vec<_>>()
        .iter()
        .fold(String::new(),|buf, &line|{ buf + line + "\n"});

    let mut reader = ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .flexible(true)
        .from_reader(transactions.as_bytes());

    for record in reader.deserialize() {
        let record: Transaction = record?;
        println!("{:?}",record);
    }

    Ok("".to_string())
}
