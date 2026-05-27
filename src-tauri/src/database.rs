use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: i64,
    pub cpu: f64,
    pub disk: f64,        // 磁盘使用率 (0-100%)
    pub ram: f64,
    pub temperature: f64, // CPU温度 (摄氏度，-1表示不可用)
    pub upload: f64,
    pub download: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawDataPoint {
    pub id: i64,
    pub timestamp: i64,
    pub cpu: f64,
    pub disk: f64,
    pub ram: f64,
    pub temperature: f64,
    pub upload: f64,
    pub download: f64,
}

impl RawDataPoint {
    pub fn get_metric(&self, metric: &str) -> f64 {
        match metric {
            "cpu" => self.cpu,
            "disk" => self.disk,
            "ram" => self.ram,
            "temperature" => self.temperature,
            "upload" => self.upload,
            "download" => self.download,
            _ => 0.0,
        }
    }
}

pub struct DatabaseManager {
    conn: Connection,
}

impl DatabaseManager {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS raw_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                cpu REAL NOT NULL,
                disk REAL NOT NULL,
                ram REAL NOT NULL,
                temperature REAL NOT NULL,
                upload REAL NOT NULL,
                download REAL NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON raw_data(timestamp)",
            [],
        )?;

        Ok(DatabaseManager { conn })
    }

    pub fn insert_raw_data(&self, metrics: &SystemMetrics) -> Result<()> {
        self.conn.execute(
            "INSERT INTO raw_data (timestamp, cpu, disk, ram, temperature, upload, download)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                metrics.timestamp,
                metrics.cpu,
                metrics.disk,
                metrics.ram,
                metrics.temperature,
                metrics.upload,
                metrics.download,
            ],
        )?;
        Ok(())
    }

    pub fn query_raw_data(&self, start_time: i64, end_time: i64) -> Result<Vec<RawDataPoint>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, cpu, disk, ram, temperature, upload, download
             FROM raw_data
             WHERE timestamp >= ?1 AND timestamp <= ?2
             ORDER BY timestamp ASC"
        )?;

        let rows = stmt.query_map(params![start_time, end_time], |row| {
            Ok(RawDataPoint {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                cpu: row.get(2)?,
                disk: row.get(3)?,
                ram: row.get(4)?,
                temperature: row.get(5)?,
                upload: row.get(6)?,
                download: row.get(7)?,
            })
        })?;

        let mut data = Vec::new();
        for row in rows {
            data.push(row?);
        }

        Ok(data)
    }

    pub fn get_latest_data(&self) -> Result<Option<SystemMetrics>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, cpu, disk, ram, temperature, upload, download
             FROM raw_data
             ORDER BY timestamp DESC
             LIMIT 1"
        )?;

        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            Ok(Some(SystemMetrics {
                timestamp: row.get(0)?,
                cpu: row.get(1)?,
                disk: row.get(2)?,
                ram: row.get(3)?,
                temperature: row.get(4)?,
                upload: row.get(5)?,
                download: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_database_operations() {
        let test_db = "/tmp/test_macoscandles.db";
        let _ = fs::remove_file(test_db);

        let db = DatabaseManager::new(Path::new(test_db)).unwrap();

        let metrics = SystemMetrics {
            timestamp: 1000,
            cpu: 50.0,
            gpu: -1.0,
            ram: 60.0,
            battery: 80.0,
            upload: 100.0,
            download: 200.0,
        };

        db.insert_raw_data(&metrics).unwrap();

        let data = db.query_raw_data(0, 2000).unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].cpu, 50.0);

        let latest = db.get_latest_data().unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().cpu, 50.0);

        let _ = fs::remove_file(test_db);
    }
}
