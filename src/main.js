// 使用Tauri的全局API
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

class ChartView {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        this.chart = null;
        this.candlestickSeries = null;
    }

    init() {
        // 等待LightweightCharts加载
        if (!window.LightweightCharts) {
            throw new Error('LightweightCharts not loaded');
        }

        const { createChart } = window.LightweightCharts;

        this.chart = createChart(this.container, {
            width: this.container.clientWidth,
            height: this.container.clientHeight,
            layout: {
                background: { color: '#1e1e1e' },
                textColor: '#d1d4dc',
            },
            grid: {
                vertLines: { color: '#2d2d2d' },
                horzLines: { color: '#2d2d2d' },
            },
            timeScale: {
                timeVisible: true,
                secondsVisible: true,
            },
            handleScroll: {
                mouseWheel: true,
                pressedMouseMove: true,
                horzTouchDrag: true,
                vertTouchDrag: true,
            },
            handleScale: {
                axisPressedMouseMove: true,
                mouseWheel: true,
                pinch: true,
            },
        });

        this.candlestickSeries = this.chart.addCandlestickSeries({
            upColor: '#26a69a',
            downColor: '#ef5350',
            borderVisible: false,
            wickUpColor: '#26a69a',
            wickDownColor: '#ef5350',
        });

        window.addEventListener('resize', () => {
            this.chart.applyOptions({
                width: this.container.clientWidth,
                height: this.container.clientHeight,
            });
        });

        console.log('Chart initialized successfully');
    }

    setData(klineData) {
        if (!klineData || klineData.length === 0) {
            return;
        }

        const formattedData = klineData.map(k => ({
            time: k.time,
            open: k.open,
            high: k.high,
            low: k.low,
            close: k.close,
        }));

        console.log('Setting data to chart:', formattedData.length, 'points');
        this.candlestickSeries.setData(formattedData);
    }

    updateData(klinePoint) {
        this.candlestickSeries.update({
            time: klinePoint.time,
            open: klinePoint.open,
            high: klinePoint.high,
            low: klinePoint.low,
            close: klinePoint.close,
        });
    }

    clear() {
        this.candlestickSeries.setData([]);
    }

    zoomIn() {
        const timeScale = this.chart.timeScale();
        const logicalRange = timeScale.getVisibleLogicalRange();
        if (logicalRange) {
            const center = (logicalRange.from + logicalRange.to) / 2;
            const newRange = (logicalRange.to - logicalRange.from) * 0.7; // 缩小范围30%
            timeScale.setVisibleLogicalRange({
                from: center - newRange / 2,
                to: center + newRange / 2,
            });
        }
    }

    zoomOut() {
        const timeScale = this.chart.timeScale();
        const logicalRange = timeScale.getVisibleLogicalRange();
        if (logicalRange) {
            const center = (logicalRange.from + logicalRange.to) / 2;
            const newRange = (logicalRange.to - logicalRange.from) * 1.3; // 扩大范围30%
            timeScale.setVisibleLogicalRange({
                from: center - newRange / 2,
                to: center + newRange / 2,
            });
        }
    }

    resetZoom() {
        this.chart.timeScale().fitContent();
    }
}

class App {
    constructor() {
        this.chart = null;
        this.currentIndicator = 'cpu';
        this.currentPeriod = '1min';
        this.currentKLines = [];  // 存储当前的K线数据
        this.lastUpdateTime = 0;   // 上次更新的时间戳
    }

    async init() {
        try {
            console.log('Initializing app...');
            console.log('LightweightCharts available:', !!window.LightweightCharts);

            // 等待LightweightCharts加载
            if (!window.LightweightCharts) {
                throw new Error('LightweightCharts not loaded from CDN');
            }

            this.chart = new ChartView('chart');
            this.chart.init();

            this.setupEventListeners();
            await this.startCollection();
            await this.loadHistoricalData();
            await this.setupRealtimeUpdates();
            console.log('App initialized successfully');
        } catch (error) {
            console.error('Failed to initialize app:', error);
            this.showError('应用初始化失败: ' + error.message);
        }
    }

    setupEventListeners() {
        document.querySelectorAll('.indicator-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const indicator = e.target.dataset.indicator;
                this.switchIndicator(indicator);
            });
        });

        // 所有周期按钮都可用
        document.querySelectorAll('.period-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const period = e.target.dataset.period;
                this.switchPeriod(period);
            });
        });

        // 缩放控制按钮
        document.getElementById('zoom-in').addEventListener('click', () => {
            this.chart.zoomIn();
        });

        document.getElementById('zoom-out').addEventListener('click', () => {
            this.chart.zoomOut();
        });

        document.getElementById('zoom-reset').addEventListener('click', () => {
            this.chart.resetZoom();
        });
    }

    async startCollection() {
        try {
            await invoke('start_collection');
            console.log('Data collection started');
        } catch (error) {
            console.error('Failed to start collection:', error);
            this.showError('启动数据采集失败');
        }
    }

    async loadHistoricalData() {
        this.showLoading();

        try {
            const endTime = Math.floor(Date.now() / 1000);
            const startTime = 0;  // 从最早的数据开始

            console.log('Loading historical data:', {
                indicator: this.currentIndicator,
                period: this.currentPeriod,
                startTime,
                endTime
            });

            const data = await invoke('get_historical_data', {
                indicator: this.currentIndicator,
                period: this.currentPeriod,
                startTime,
                endTime,
            });

            console.log('Received data:', data);

            if (!data || data.length === 0) {
                console.log('No data available yet');
                this.showNoData();
            } else {
                console.log(`Setting ${data.length} K-lines to chart`);
                this.currentKLines = data;  // 保存当前K线数据
                this.chart.setData(data);
                this.hideMessages();
            }
        } catch (error) {
            console.error('Failed to load data:', error);
            this.showError('加载数据失败: ' + error);
        }
    }

    async switchIndicator(indicator) {
        console.log('Switching to indicator:', indicator);

        document.querySelectorAll('.indicator-btn').forEach(btn => {
            btn.classList.toggle('active', btn.dataset.indicator === indicator);
        });

        this.currentIndicator = indicator;
        await this.loadHistoricalData();
    }

    async switchPeriod(period) {
        console.log('Switching to period:', period);

        document.querySelectorAll('.period-btn').forEach(btn => {
            btn.classList.toggle('active', btn.dataset.period === period);
        });

        this.currentPeriod = period;
        await this.loadHistoricalData();
    }

    async setupRealtimeUpdates() {
        await listen('data-update', (event) => {
            const metrics = event.payload;
            console.log('Received data update:', metrics);

            // 更新K线
            this.updateKLineWithNewData(metrics);
        });
    }

    updateKLineWithNewData(metrics) {
        // 如果没有当前K线数据，不处理
        if (!this.currentKLines || this.currentKLines.length === 0) {
            return;
        }

        // 获取周期的秒数
        const periodSeconds = this.getPeriodSeconds(this.currentPeriod);

        // 计算当前数据点属于哪个K线窗口
        const windowStart = Math.floor(metrics.timestamp / periodSeconds) * periodSeconds;

        // 获取当前指标的值
        const value = metrics[this.currentIndicator];
        if (value === undefined) {
            return;
        }

        // 获取最后一根K线
        const lastKLine = this.currentKLines[this.currentKLines.length - 1];

        if (lastKLine && lastKLine.time === windowStart) {
            // 更新当前K线
            lastKLine.high = Math.max(lastKLine.high, value);
            lastKLine.low = Math.min(lastKLine.low, value);
            lastKLine.close = value;

            console.log('Updating current K-line:', lastKLine);
            this.chart.updateData(lastKLine);
        } else {
            // 创建新K线
            const newKLine = {
                time: windowStart,
                open: value,
                high: value,
                low: value,
                close: value,
            };

            this.currentKLines.push(newKLine);
            console.log('Creating new K-line:', newKLine);
            this.chart.updateData(newKLine);
        }
    }

    getPeriodSeconds(period) {
        const periodMap = {
            '5s': 5,
            '1min': 60,
            '5min': 300,
            '15min': 900,
            '1h': 3600,
        };
        return periodMap[period] || 60;
    }

    showLoading() {
        document.getElementById('loading').classList.remove('hidden');
        document.getElementById('error').classList.add('hidden');
        document.getElementById('no-data').classList.add('hidden');
    }

    showError(message) {
        const errorEl = document.getElementById('error');
        errorEl.textContent = message;
        errorEl.classList.remove('hidden');
        document.getElementById('loading').classList.add('hidden');
        document.getElementById('no-data').classList.add('hidden');
    }

    showNoData() {
        document.getElementById('no-data').classList.remove('hidden');
        document.getElementById('loading').classList.add('hidden');
        document.getElementById('error').classList.add('hidden');
    }

    hideMessages() {
        document.getElementById('loading').classList.add('hidden');
        document.getElementById('error').classList.add('hidden');
        document.getElementById('no-data').classList.add('hidden');
    }
}

// 等待LightweightCharts加载后再初始化
function initApp() {
    if (window.LightweightCharts) {
        console.log('DOM loaded, creating app...');
        const app = new App();
        app.init().catch(err => {
            console.error('App initialization failed:', err);
        });
    } else {
        console.log('Waiting for LightweightCharts to load...');
        setTimeout(initApp, 100);
    }
}

document.addEventListener('DOMContentLoaded', initApp);

