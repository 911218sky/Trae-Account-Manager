export type ErrorType = 'network' | 'cache' | 'render' | 'performance';

export interface ErrorLog {
  timestamp: number;
  type: ErrorType;
  message: string;
  stack?: string;
  context?: Record<string, any>;
}

export class ErrorMonitor {
  private logs: ErrorLog[] = [];
  private maxLogs: number;

  constructor(maxLogs: number = 100) {
    this.maxLogs = maxLogs;
  }

  logError(
    type: ErrorType,
    error: Error,
    context?: Record<string, any>
  ): void {
    const log: ErrorLog = {
      timestamp: Date.now(),
      type,
      message: error.message,
      stack: error.stack,
      context,
    };

    this.logs.push(log);

    // 限制日誌數量
    if (this.logs.length > this.maxLogs) {
      this.logs.shift();
    }

    // 輸出到控制台
    console.error(`[ErrorMonitor] ${type.toUpperCase()} 錯誤:`, {
      message: error.message,
      context,
      stack: error.stack,
    });
  }

  getRecentErrors(count: number = 10): ErrorLog[] {
    return this.logs.slice(-count);
  }

  getErrorsByType(type: ErrorType): ErrorLog[] {
    return this.logs.filter((log) => log.type === type);
  }

  clearLogs(): void {
    this.logs = [];
  }

  getErrorCount(): number {
    return this.logs.length;
  }

  getErrorCountByType(type: ErrorType): number {
    return this.logs.filter((log) => log.type === type).length;
  }

  exportLogs(): string {
    return JSON.stringify(this.logs, null, 2);
  }
}

// 單例實例
export const errorMonitor = new ErrorMonitor();
