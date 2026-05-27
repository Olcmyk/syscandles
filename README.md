# macOS Candles

[English](#english) | [中文](#中文)
<img width="1194" height="800" alt="图片" src="https://github.com/user-attachments/assets/7f5f4b16-c5a5-4a82-a180-c2bb6d9ad58e" />

## English

A macOS system monitoring application that visualizes system metrics as candlestick charts.

### Features

Real-time monitoring and candlestick chart visualization of system metrics:
- CPU Usage
- CPU Temperature
- Memory Usage
- Disk Usage
- Network Upload Speed
- Network Download Speed

### Tech Stack

- **Tauri 2** - Cross-platform desktop application framework
- **Rust** - System monitoring data collection
- **JavaScript** - Frontend interaction
- **Lightweight Charts** - Candlestick chart rendering

### Database Storage

The application stores monitoring data in a SQLite database located at:

**macOS:** `~/Library/Application Support/com.macoscandles.app/data.db`

This database contains historical system metrics collected at regular intervals. You can delete this file to clear all historical data.

---

## 中文

一个将 macOS 系统监控数据以 K 线图形式可视化的应用。

### 功能

实时监控并以 K 线图（蜡烛图）展示系统指标：
- CPU 使用率
- CPU 温度
- 内存使用情况
- 磁盘使用情况
- 网络上传速度
- 网络下载速度

### 技术栈

- **Tauri 2** - 跨平台桌面应用框架
- **Rust** - 系统监控数据采集
- **JavaScript** - 前端交互
- **Lightweight Charts** - K 线图渲染

### 数据库存储

应用程序将监控数据存储在 SQLite 数据库中，位置为：

**macOS:** `~/Library/Application Support/com.macoscandles.app/data.db`

该数据库包含定期收集的历史系统指标数据。您可以删除此文件以清除所有历史数据。

---

## License

MIT License - Copyright (c) 2026 Olcmyk
