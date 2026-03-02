import { useState, useCallback } from 'react';

export interface SelectionManager {
  selectedIds: Set<string>;
  lastSelectedId: string | null;
  toggleSelection: (id: string, shiftKey?: boolean, allIds?: string[]) => void;
  selectAll: (ids: string[]) => void;
  deselectAll: () => void;
  isSelected: (id: string) => boolean;
  getSelectionCount: () => number;
}

/**
 * Selection Manager Hook.
 * 
 * Manages account selection state with support for single selection,
 * select all, and Shift key range selection.
 */
export function useSelectionManager(): SelectionManager {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [lastSelectedId, setLastSelectedId] = useState<string | null>(null);

  /**
   * Toggle selection state for a single account.
   * 
   * Supports Shift key for range selection.
   */
  const toggleSelection = useCallback(
    (id: string, shiftKey = false, allIds: string[] = []) => {
      if (shiftKey && lastSelectedId && allIds.length > 0) {
        // Shift key range selection
        const lastIndex = allIds.indexOf(lastSelectedId);
        const currentIndex = allIds.indexOf(id);

        if (lastIndex !== -1 && currentIndex !== -1) {
          const start = Math.min(lastIndex, currentIndex);
          const end = Math.max(lastIndex, currentIndex);
          const rangeIds = allIds.slice(start, end + 1);

          setSelectedIds((prev) => {
            const next = new Set(prev);
            rangeIds.forEach((rangeId) => next.add(rangeId));
            return next;
          });
          return;
        }
      }

      // Regular toggle
      setSelectedIds((prev) => {
        const next = new Set(prev);
        if (next.has(id)) {
          next.delete(id);
        } else {
          next.add(id);
        }
        return next;
      });
      setLastSelectedId(id);
    },
    [lastSelectedId]
  );

  /**
   * Select all visible accounts.
   */
  const selectAll = useCallback((ids: string[]) => {
    setSelectedIds(new Set(ids));
    if (ids.length > 0) {
      setLastSelectedId(ids[ids.length - 1]);
    }
  }, []);

  /**
   * Deselect all accounts.
   */
  const deselectAll = useCallback(() => {
    setSelectedIds(new Set());
    setLastSelectedId(null);
  }, []);

  /**
   * Check if an account is selected.
   */
  const isSelected = useCallback(
    (id: string) => {
      return selectedIds.has(id);
    },
    [selectedIds]
  );

  /**
   * Get the count of selected accounts.
   */
  const getSelectionCount = useCallback(() => {
    return selectedIds.size;
  }, [selectedIds]);

  return {
    selectedIds,
    lastSelectedId,
    toggleSelection,
    selectAll,
    deselectAll,
    isSelected,
    getSelectionCount,
  };
}
