use tauri_app_lib::{database, aggregator};

#[tokio::main]
async fn main() {
    let db_path = std::path::PathBuf::from(
        std::env::var("HOME").unwrap() + "/Library/Application Support/com.macoscandles.app/data.db"
    );

    let db = database::DatabaseManager::new(&db_path).unwrap();

    let end_time = chrono::Utc::now().timestamp();
    let start_time = end_time - 3600; // Last 1 hour

    println!("Querying data from {} to {}", start_time, end_time);

    let raw_data = db.query_raw_data(start_time, end_time).unwrap();
    println!("Found {} raw data points", raw_data.len());

    if raw_data.len() > 0 {
        println!("First point: timestamp={}, cpu={}, ram={}",
            raw_data[0].timestamp, raw_data[0].cpu, raw_data[0].ram);
        println!("Last point: timestamp={}, cpu={}, ram={}",
            raw_data[raw_data.len()-1].timestamp,
            raw_data[raw_data.len()-1].cpu,
            raw_data[raw_data.len()-1].ram);
    }

    let klines = aggregator::aggregate_to_kline(raw_data, 60, "cpu");
    println!("\nAggregated to {} K-lines (1-minute period)", klines.len());

    if klines.len() > 0 {
        println!("First K-line: time={}, open={}, high={}, low={}, close={}",
            klines[0].time, klines[0].open, klines[0].high, klines[0].low, klines[0].close);
        println!("Last K-line: time={}, open={}, high={}, low={}, close={}",
            klines[klines.len()-1].time,
            klines[klines.len()-1].open,
            klines[klines.len()-1].high,
            klines[klines.len()-1].low,
            klines[klines.len()-1].close);
    }
}
