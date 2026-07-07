import { useMemo, useRef, useState } from "react";
import type { DatabaseTable } from "@unfour/command-client";
import { databaseTableTreeId } from "../model/database-tree";
import { defaultSql } from "../model/database-state";
import {
  emptyTableQuery,
  type DatabaseQueryWorkspaceTab,
  type DatabaseTableWorkspaceTab,
  type DatabaseWorkspaceTab,
  type DatabaseWorkspaceTabId,
  type TableSegment,
} from "../model/types";

type QueryTabInput = {
  catalog?: string | null;
  connectionId?: string | null;
  schema?: string | null;
  sql?: string;
};

type DatabaseTabsOptions = {
  formatQueryTitle?: (index: number) => string;
};

type QueryTabPatch =
  Partial<Omit<DatabaseQueryWorkspaceTab, "id" | "kind">> |
  ((tab: DatabaseQueryWorkspaceTab) => Partial<Omit<DatabaseQueryWorkspaceTab, "id" | "kind">>);

type TableTabPatch =
  Partial<Omit<DatabaseTableWorkspaceTab, "id" | "kind">> |
  ((tab: DatabaseTableWorkspaceTab) => Partial<Omit<DatabaseTableWorkspaceTab, "id" | "kind">>);

type DatabaseTabsState = {
  activeTabId: DatabaseWorkspaceTabId;
  tabs: DatabaseWorkspaceTab[];
};

export function databaseTableTabId(connectionId: string, table: DatabaseTable) {
  return `database-tab:${databaseTableTreeId(connectionId, table)}`;
}

export function useDatabaseTabs(options: DatabaseTabsOptions = {}) {
  const formatQueryTitle = options.formatQueryTitle ?? defaultQueryTitle;
  const nextQueryIndexRef = useRef(2);
  const [state, setState] = useState<DatabaseTabsState>(() => {
    const tab = createQueryTab(1, {}, formatQueryTitle);
    return { activeTabId: tab.id, tabs: [tab] };
  });

  const activeTab = useMemo(
    () => state.tabs.find((tab) => tab.id === state.activeTabId) ?? state.tabs[0] ?? null,
    [state.activeTabId, state.tabs],
  );

  function openQueryTab(input: QueryTabInput = {}) {
    const tab = createQueryTab(nextQueryIndexRef.current, input, formatQueryTitle);
    nextQueryIndexRef.current += 1;
    setState((current) => ({
      activeTabId: tab.id,
      tabs: [...current.tabs, tab],
    }));
    return tab.id;
  }

  function openTableTab(connectionId: string, table: DatabaseTable, segment: TableSegment = "data") {
    const tabId = databaseTableTabId(connectionId, table);
    setState((current) => {
      const exists = current.tabs.some((tab) => tab.id === tabId);
      return {
        activeTabId: tabId,
        tabs: exists
          ? current.tabs.map((tab) =>
              tab.id === tabId && tab.kind === "table" ? { ...tab, segment } : tab,
            )
          : [...current.tabs, createTableTab(tabId, connectionId, table, segment)],
      };
    });
    return tabId;
  }

  function setActiveTabId(tabId: DatabaseWorkspaceTabId) {
    setState((current) =>
      current.tabs.some((tab) => tab.id === tabId)
        ? { ...current, activeTabId: tabId }
        : current,
    );
  }

  function closeTab(tabId: DatabaseWorkspaceTabId) {
    // Decide + allocate the replacement index OUTSIDE the setState updater.
    // React.StrictMode double-invokes state updaters in dev; mutating
    // nextQueryIndexRef inside the updater used to increment it twice and skip a
    // query number (closing the lone "Query 1" produced "Query 3"). Event
    // handlers are not double-invoked, so advancing here is safe and runs once.
    const needsFallback = state.tabs.filter((tab) => tab.id !== tabId).length === 0;
    const fallbackTab = needsFallback
      ? createQueryTab(nextQueryIndexRef.current, {}, formatQueryTitle)
      : null;
    if (fallbackTab) {
      nextQueryIndexRef.current += 1;
    }
    setState((current) => {
      const tabs = current.tabs.filter((tab) => tab.id !== tabId);
      const nextTabs = tabs.length ? tabs : fallbackTab ? [fallbackTab] : [createQueryTab(nextQueryIndexRef.current, {}, formatQueryTitle)];
      const activeTabId =
        current.activeTabId === tabId
          ? (nextTabs[Math.max(0, current.tabs.findIndex((tab) => tab.id === tabId) - 1)] ?? nextTabs[0]).id
          : current.activeTabId;
      return {
        activeTabId: nextTabs.some((tab) => tab.id === activeTabId) ? activeTabId : nextTabs[0].id,
        tabs: nextTabs,
      };
    });
  }

  function reorderTabs(fromIndex: number, toIndex: number) {
    setState((current) => {
      if (
        fromIndex < 0 ||
        toIndex < 0 ||
        fromIndex >= current.tabs.length ||
        toIndex >= current.tabs.length ||
        fromIndex === toIndex
      ) {
        return current;
      }
      const tabs = [...current.tabs];
      const [moved] = tabs.splice(fromIndex, 1);
      tabs.splice(toIndex, 0, moved);
      return { ...current, tabs };
    });
  }

  function updateQueryTab(tabId: DatabaseWorkspaceTabId, patch: QueryTabPatch) {
    setState((current) => ({
      ...current,
      tabs: current.tabs.map((tab) => {
        if (tab.id !== tabId || tab.kind !== "query") {
          return tab;
        }
        const nextPatch = typeof patch === "function" ? patch(tab) : patch;
        return { ...tab, ...nextPatch };
      }),
    }));
  }

  function updateTableTab(tabId: DatabaseWorkspaceTabId, patch: TableTabPatch) {
    setState((current) => ({
      ...current,
      tabs: current.tabs.map((tab) => {
        if (tab.id !== tabId || tab.kind !== "table") {
          return tab;
        }
        const nextPatch = typeof patch === "function" ? patch(tab) : patch;
        return { ...tab, ...nextPatch };
      }),
    }));
  }

  function removeConnectionTabs(connectionId: string) {
    // See closeTab: allocate the replacement index outside the updater so the
    // counter is not double-incremented under React.StrictMode.
    const needsFallback = state.tabs.filter(
      (tab) => !(tab.kind === "table" && tab.connectionId === connectionId),
    ).length === 0;
    const fallbackTab = needsFallback
      ? createQueryTab(nextQueryIndexRef.current, {}, formatQueryTitle)
      : null;
    if (fallbackTab) {
      nextQueryIndexRef.current += 1;
    }
    setState((current) => {
      const tabs = current.tabs
        .filter((tab) => !(tab.kind === "table" && tab.connectionId === connectionId))
        .map((tab) =>
          tab.kind === "query" && tab.connectionId === connectionId
            ? {
                ...tab,
                catalog: null,
                connectionId: null,
                error: null,
                pendingConfirmation: false,
                schema: null,
              }
            : tab,
        );
      const nextTabs = tabs.length ? tabs : fallbackTab ? [fallbackTab] : [createQueryTab(nextQueryIndexRef.current, {}, formatQueryTitle)];
      return {
        activeTabId: nextTabs.some((tab) => tab.id === current.activeTabId)
          ? current.activeTabId
          : nextTabs[0].id,
        tabs: nextTabs,
      };
    });
  }

  return {
    activeTab,
    activeTabId: state.activeTabId,
    closeTab,
    openQueryTab,
    openTableTab,
    removeConnectionTabs,
    reorderTabs,
    setActiveTabId,
    tabs: state.tabs,
    updateQueryTab,
    updateTableTab,
  };
}

function defaultQueryTitle(index: number) {
  return `Query ${index}`;
}

function createQueryTab(
  index: number,
  input: QueryTabInput = {},
  formatQueryTitle: (index: number) => string = defaultQueryTitle,
): DatabaseQueryWorkspaceTab {
  return {
    catalog: input.catalog ?? null,
    connectionId: input.connectionId ?? null,
    error: null,
    id: `database-query-${index}`,
    kind: "query",
    pendingConfirmation: false,
    result: null,
    resultTab: "results",
    schema: input.schema ?? null,
    sql: input.sql ?? defaultSql,
    title: formatQueryTitle(index),
  };
}

function createTableTab(
  id: string,
  connectionId: string,
  table: DatabaseTable,
  segment: TableSegment,
): DatabaseTableWorkspaceTab {
  return {
    connectionId,
    error: null,
    id,
    kind: "table",
    queryResult: null,
    segment,
    structureTab: "ddl",
    table,
    tableQuery: { ...emptyTableQuery },
    tableView: null,
    title: table.name,
  };
}
