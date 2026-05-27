use crate::database::RawDataPoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KLineData {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn period_to_seconds(period: &str) -> Result<i64, String> {
    match period {
        "5s" => Ok(5),
        "1min" => Ok(60),
        "5min" => Ok(300),
        "15min" => Ok(900),
        "1h" => Ok(3600),
        _ => Err(format!("Unsupported period: {}", period)),
    }
}

pub fn aggregate_to_kline(
    raw_data: Vec<RawDataPoint>,
    period_seconds: i64,
    metric: &str,
) -> Vec<KLineData> {
    if raw_data.is_empty() {
        return Vec::new();
    }

    let mut klines = Vec::new();
    let mut current_window_start: Option<i64> = None;
    let mut window_data: Vec<f64> = Vec::new();

    for point in raw_data {
        let window_start = (point.timestamp / period_seconds) * period_seconds;

        if let Some(current_start) = current_window_start {
            if window_start != current_start {
                // New window, finalize previous window
                if !window_data.is_empty() {
                    klines.push(create_kline(&window_data, current_start));
                }
                window_data.clear();
                current_window_start = Some(window_start);
            }
        } else {
            // First window
            current_window_start = Some(window_start);
        }

        let value = point.get_metric(metric);
        window_data.push(value);
    }

    // Handle last window
    if !window_data.is_empty() {
        if let Some(start) = current_window_start {
            klines.push(create_kline(&window_data, start));
        }
    }

    klines
}

fn create_kline(values: &[f64], timestamp: i64) -> KLineData {
    let open = values.first().copied().unwrap_or(0.0);
    let close = values.last().copied().unwrap_or(0.0);
    let high = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let low = values.iter().copied().fold(f64::INFINITY, f64::min);

    KLineData {
        time: timestamp,
        open,
        high,
        low,
        close,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_period_to_seconds() {
        assert_eq!(period_to_seconds("5s").unwrap(), 5);
        assert_eq!(period_to_seconds("1min").unwrap(), 60);
        assert_eq!(period_to_seconds("5min").unwrap(), 300);
        assert_eq!(period_to_seconds("15min").unwrap(), 900);
        assert_eq!(period_to_seconds("1h").unwrap(), 3600);
        assert!(period_to_seconds("invalid").is_err());
    }

    #[test]
    fn test_aggregate_to_kline() {
        // Create 12 data points (1 minute of 5-second data)
        let mut raw_data = Vec::new();
        for i in 0..12 {
            raw_data.push(RawDataPoint {
                id: i,
                timestamp: 1000 + i * 5,
                cpu: 50.0 + (i as f64),
                gpu: -1.0,
                ram: 60.0,
                battery: 80.0,
                upload: 100.0,
                download: 200.0,
            });
        }

        // Aggregate to 1-minute K-lines
        let klines = aggregate_to_kline(raw_data, 60, "cpu");

        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].open, 50.0);
        assert_eq!(klines[0].close, 61.0);
        assert_eq!(klines[0].high, 61.0);
        assert_eq!(klines[0].low, 50.0);
    }

    #[test]
    fn test_empty_data() {
        let klines = aggregate_to_kline(Vec::new(), 60, "cpu");
        assert_eq!(klines.len(), 0);
    }

    #[test]
    fn test_single_point() {
        let raw_data = vec![RawDataPoint {
            id: 1,
            timestamp: 1000,
            cpu: 50.0,
            gpu: -1.0,
            ram: 60.0,
            battery: 80.0,
            upload: 100.0,
            download: 200.0,
        }];

        let klines = aggregate_to_kline(raw_data, 60, "cpu");

        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].open, 50.0);
        assert_eq!(klines[0].close, 50.0);
        assert_eq!(klines[0].high, 50.0);
        assert_eq!(klines[0].low, 50.0);
    }
}
