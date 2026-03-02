import { useEffect, useRef } from 'react';
import { performanceMonitor, PerformanceMetrics } from '../services/performanceMonitor';

export function usePerformanceMonitor(enabled: boolean = true) {
  const frameIdRef = useRef<number | undefined>(undefined);
  const isMonitoringRef = useRef(false);

  useEffect(() => {
    if (!enabled) {
      return;
    }

    // 開始監控主線程阻塞時間
    performanceMonitor.measureMainThreadBlockTime();

    // 開始 FPS 監控
    const measureFPS = () => {
      performanceMonitor.updateFPS();
      frameIdRef.current = requestAnimationFrame(measureFPS);
    };

    if (!isMonitoringRef.current) {
      isMonitoringRef.current = true;
      frameIdRef.current = requestAnimationFrame(measureFPS);
    }

    return () => {
      if (frameIdRef.current) {
        cancelAnimationFrame(frameIdRef.current);
      }
      isMonitoringRef.current = false;
    };
  }, [enabled]);

  const startMeasure = (name: string) => {
    performanceMonitor.startMeasure(name);
  };

  const endMeasure = (name: string): number => {
    return performanceMonitor.endMeasure(name);
  };

  const recordCacheHit = () => {
    performanceMonitor.recordCacheHit();
  };

  const recordCacheMiss = () => {
    performanceMonitor.recordCacheMiss();
  };

  const getMetrics = (): PerformanceMetrics => {
    return performanceMonitor.getMetrics();
  };

  const logMetrics = () => {
    performanceMonitor.logMetrics();
  };

  const reset = () => {
    performanceMonitor.reset();
  };

  return {
    startMeasure,
    endMeasure,
    recordCacheHit,
    recordCacheMiss,
    getMetrics,
    logMetrics,
    reset,
  };
}
