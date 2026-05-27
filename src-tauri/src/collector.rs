use crate::database::{DatabaseManager, SystemMetrics};
use std::sync::{Arc, Mutex};
use sysinfo::{System, Networks, Disks, Components};
use tauri::Emitter;
use tokio::time::{interval, Duration};

pub struct DataCollector {
    db_manager: Arc<Mutex<DatabaseManager>>,
    is_running: Arc<Mutex<bool>>,
    prev_network_stats: Arc<Mutex<Option<NetworkStats>>>,
}

#[derive(Clone)]
struct NetworkStats {
    total_received: u64,
    total_transmitted: u64,
    timestamp: i64,
}

impl DataCollector {
    pub fn new(db_manager: Arc<Mutex<DatabaseManager>>) -> Self {
        DataCollector {
            db_manager,
            is_running: Arc::new(Mutex::new(false)),
            prev_network_stats: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start_collection_loop(self: Arc<Self>, app_handle: tauri::AppHandle) {
        {
            let mut running = self.is_running.lock().unwrap();
            *running = true;
        }

        let mut interval_timer = interval(Duration::from_secs(5));

        loop {
            interval_timer.tick().await;

            let is_running = {
                let running = self.is_running.lock().unwrap();
                *running
            };

            if !is_running {
                break;
            }

            match self.collect_metrics() {
                Ok(metrics) => {
                    // Save to database
                    if let Ok(db) = self.db_manager.lock() {
                        if let Err(e) = db.insert_raw_data(&metrics) {
                            eprintln!("Failed to insert data: {}", e);
                        }
                    }

                    // Emit event to frontend
                    if let Err(e) = app_handle.emit("data-update", &metrics) {
                        eprintln!("Failed to emit event: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to collect metrics: {}", e);
                }
            }
        }
    }

    pub fn stop(&self) {
        let mut running = self.is_running.lock().unwrap();
        *running = false;
    }

    fn collect_metrics(&self) -> Result<SystemMetrics, String> {
        let mut sys = System::new_all();
        sys.refresh_all();

        // CPU usage
        let cpu = sys.global_cpu_info().cpu_usage() as f64;
        let cpu = if cpu.is_nan() || cpu < 0.0 { 0.0 } else { cpu.min(100.0) };

        // RAM usage
        let ram = (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0;
        let ram = if ram.is_nan() || ram < 0.0 { 0.0 } else { ram.min(100.0) };

        // Disk usage
        let disk = self.get_disk_usage();

        // CPU temperature
        let temperature = self.get_cpu_temperature();

        // Network speed (KB/s)
        let (upload, download) = self.calculate_network_speed(&sys);

        let timestamp = chrono::Utc::now().timestamp();

        Ok(SystemMetrics {
            timestamp,
            cpu,
            disk,
            ram,
            temperature,
            upload,
            download,
        })
    }

    fn get_disk_usage(&self) -> f64 {
        let disks = Disks::new_with_refreshed_list();

        // 获取主磁盘（通常是第一个或根目录）
        for disk in &disks {
            let mount_point = disk.mount_point().to_string_lossy();
            if mount_point == "/" {
                let total = disk.total_space() as f64;
                let available = disk.available_space() as f64;
                let used = total - available;
                let usage_percent = (used / total) * 100.0;
                return if usage_percent.is_nan() { 0.0 } else { usage_percent.min(100.0) };
            }
        }

        // 如果没找到根目录，使用第一个磁盘
        if let Some(disk) = disks.first() {
            let total = disk.total_space() as f64;
            let available = disk.available_space() as f64;
            let used = total - available;
            let usage_percent = (used / total) * 100.0;
            return if usage_percent.is_nan() { 0.0 } else { usage_percent.min(100.0) };
        }

        0.0
    }

    fn get_cpu_temperature(&self) -> f64 {
        let components = Components::new_with_refreshed_list();

        // 优先查找性能核心温度（pACC）
        for component in &components {
            let label = component.label();
            if label.contains("pACC") && label.contains("Temp") {
                let temp = component.temperature() as f64;
                if temp > 0.0 && temp < 150.0 {
                    return temp;
                }
            }
        }

        // 其次查找SOC温度
        for component in &components {
            let label = component.label();
            if label.contains("SOC") && label.contains("Die") && label.contains("Temp") {
                let temp = component.temperature() as f64;
                if temp > 0.0 && temp < 150.0 {
                    return temp;
                }
            }
        }

        // 最后查找任何包含"Die"或"MTR"的温度传感器
        for component in &components {
            let label = component.label();
            if (label.contains("Die") || label.contains("MTR")) && label.contains("Temp") {
                let temp = component.temperature() as f64;
                if temp > 0.0 && temp < 150.0 {
                    return temp;
                }
            }
        }

        // 如果都没找到，返回-1表示不可用
        -1.0
    }

    fn calculate_network_speed(&self, _sys: &System) -> (f64, f64) {
        let networks = Networks::new_with_refreshed_list();

        let mut total_received: u64 = 0;
        let mut total_transmitted: u64 = 0;

        for (_interface_name, network) in networks.iter() {
            total_received += network.total_received();
            total_transmitted += network.total_transmitted();
        }

        let current_timestamp = chrono::Utc::now().timestamp();
        let current_stats = NetworkStats {
            total_received,
            total_transmitted,
            timestamp: current_timestamp,
        };

        let mut prev_stats_guard = self.prev_network_stats.lock().unwrap();

        let (upload_kbps, download_kbps) = if let Some(prev) = prev_stats_guard.as_ref() {
            let time_diff = (current_timestamp - prev.timestamp) as f64;

            if time_diff > 0.0 {
                let received_diff = total_received.saturating_sub(prev.total_received) as f64;
                let transmitted_diff = total_transmitted.saturating_sub(prev.total_transmitted) as f64;

                let download_kbps = (received_diff / 1024.0) / time_diff;
                let upload_kbps = (transmitted_diff / 1024.0) / time_diff;

                // Detect anomalies (> 1GB/s is suspicious)
                let max_speed = 1024.0 * 1024.0; // 1GB/s in KB/s
                let download_kbps = if download_kbps > max_speed { 0.0 } else { download_kbps };
                let upload_kbps = if upload_kbps > max_speed { 0.0 } else { upload_kbps };

                (upload_kbps, download_kbps)
            } else {
                (0.0, 0.0)
            }
        } else {
            // First collection, no previous data
            (0.0, 0.0)
        };

        *prev_stats_guard = Some(current_stats);

        (upload_kbps, download_kbps)
    }
}
