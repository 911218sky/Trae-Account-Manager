import { useState, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import * as api from '../api';

export interface ExportOptions {
  accountIds: string[];
  format: 'csv' | 'json';
}

export interface ExportState {
  isExporting: boolean;
  progress: api.ExportProgress | null;
  error: string | null;
}

/**
 * 匯出控制器 Hook
 * 管理匯出流程、進度監聽和錯誤處理
 */
export function useExportController() {
  const [state, setState] = useState<ExportState>({
    isExporting: false,
    progress: null,
    error: null,
  });

  /**
   * 開始匯出
   */
  const startExport = useCallback(async (options: ExportOptions): Promise<string> => {
    // 驗證選擇
    if (options.accountIds.length === 0) {
      throw new Error('請至少選擇一個帳號');
    }

    if (options.accountIds.length > 10000) {
      throw new Error('匯出數量超過限制（最多 10,000 筆）');
    }

    setState({
      isExporting: true,
      progress: {
        current: 0,
        total: options.accountIds.length,
        percentage: 0,
        estimated_time_remaining: 0,
      },
      error: null,
    });

    try {
      // 監聽進度事件
      const unlisten = await listen<api.ExportProgress>('export-progress', (event) => {
        setState((prev) => ({
          ...prev,
          progress: event.payload,
        }));
      });

      // 呼叫後端 API
      const response = await api.exportSelectedAccounts(
        options.accountIds,
        options.format
      );

      // 停止監聽
      unlisten();

      setState({
        isExporting: false,
        progress: null,
        error: null,
      });

      return response.download_url;
    } catch (error: any) {
      setState({
        isExporting: false,
        progress: null,
        error: error.message || '匯出失敗',
      });
      throw error;
    }
  }, []);

  /**
   * 重置狀態
   */
  const reset = useCallback(() => {
    setState({
      isExporting: false,
      progress: null,
      error: null,
    });
  }, []);

  return {
    ...state,
    startExport,
    reset,
  };
}
