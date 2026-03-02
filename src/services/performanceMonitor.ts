export interface PerformanceMetrics {
  firstLoadTime: number;      // First load time (ms)
  fps: number;                 // Current FPS
  mainThreadBlockTime: number; // Main thread block time (ms)
  renderTime: number;          // Render time (ms)
  cacheHitRate: number;        // Cache hit rate (0-1)
}

interface MeasureEntry {
  name: string;
  startTime: number;
  endTime?: number;
}

export class PerformanceMonitor {
  private measures: Map<string, MeasureEntry>;
  private metrics: Partial<PerformanceMetrics>;
  private fpsFrames: number[];
  private fpsLastTime: number;
  private cacheHits: number;
  private cacheMisses: number;

  constructor() {
    this.measures = new Map();
    this.metrics = {};
    this.fpsFrames = [];
    this.fpsLastTime = 0;
    this.cacheHits = 0;
    this.cacheMisses = 0;
  }

  startMeasure(name: string): void {
    this.measures.set(name, {
      name,
      startTime: performance.now(),
    });
  }

  endMeasure(name: string): number {
    const entry = this.measures.get(name);
    if (!entry) {
      console.warn(`[PerformanceMonitor] Measurement "${name}" does not exist`);
      return 0;
    }

    const endTime = performance.now();
    const duration = endTime - entry.startTime;
    
    entry.endTime = endTime;
    
    // Record specific measurements
    if (name === 'firstLoad') {
      this.metrics.firstLoadTime = duration;
    } else if (name === 'render') {
      this.metrics.renderTime = duration;
    }

    return duration;
  }

  recordCacheHit(): void {
    this.cacheHits++;
  }

  recordCacheMiss(): void {
    this.cacheMisses++;
  }

  updateFPS(): void {
    const now = performance.now();
    
    if (this.fpsLastTime === 0) {
      this.fpsLastTime = now;
      return;
    }

    const delta = now - this.fpsLastTime;
    const fps = 1000 / delta;
    
    this.fpsFrames.push(fps);
    
    // Keep only the last 60 frames
    if (this.fpsFrames.length > 60) {
      this.fpsFrames.shift();
    }
    
    this.fpsLastTime = now;
    
    // Calculate average FPS
    const avgFps = this.fpsFrames.reduce((sum, f) => sum + f, 0) / this.fpsFrames.length;
    this.metrics.fps = Math.round(avgFps);
  }

  measureMainThreadBlockTime(): void {
    if (!window.PerformanceObserver) {
      console.warn('[PerformanceMonitor] PerformanceObserver is not supported');
      return;
    }

    try {
      const observer = new PerformanceObserver((list) => {
        const entries = list.getEntries();
        let maxBlockTime = 0;
        
        entries.forEach((entry) => {
          if (entry.duration > maxBlockTime) {
            maxBlockTime = entry.duration;
          }
        });
        
        if (maxBlockTime > 0) {
          this.metrics.mainThreadBlockTime = maxBlockTime;
        }
      });

      observer.observe({ entryTypes: ['longtask'] });
    } catch (error) {
      console.warn('[PerformanceMonitor] Unable to monitor long tasks:', error);
    }
  }

  getMetrics(): PerformanceMetrics {
    // Calculate cache hit rate
    const totalCacheRequests = this.cacheHits + this.cacheMisses;
    this.metrics.cacheHitRate = totalCacheRequests > 0 
      ? this.cacheHits / totalCacheRequests 
      : 0;

    return {
      firstLoadTime: this.metrics.firstLoadTime || 0,
      fps: this.metrics.fps || 0,
      mainThreadBlockTime: this.metrics.mainThreadBlockTime || 0,
      renderTime: this.metrics.renderTime || 0,
      cacheHitRate: this.metrics.cacheHitRate || 0,
    };
  }

  reset(): void {
    this.measures.clear();
    this.metrics = {};
    this.fpsFrames = [];
    this.fpsLastTime = 0;
    this.cacheHits = 0;
    this.cacheMisses = 0;
  }

  logMetrics(): void {
    const metrics = this.getMetrics();
    console.log('[PerformanceMonitor] Performance metrics:', {
      'First Load Time': `${metrics.firstLoadTime.toFixed(2)}ms`,
      'FPS': metrics.fps,
      'Main Thread Block Time': `${metrics.mainThreadBlockTime.toFixed(2)}ms`,
      'Render Time': `${metrics.renderTime.toFixed(2)}ms`,
      'Cache Hit Rate': `${(metrics.cacheHitRate * 100).toFixed(1)}%`,
    });
  }
}

// Singleton instance
export const performanceMonitor = new PerformanceMonitor();
