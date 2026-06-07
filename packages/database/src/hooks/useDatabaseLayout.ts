import { useState } from "react";
import { defaultDatabaseTabs } from "../model/database-tabs";

export function useDatabaseLayout() {
  const [activeTabId, setActiveTabId] = useState(defaultDatabaseTabs[0].id);
  const [tabs, setTabs] = useState(defaultDatabaseTabs);
  const [resultTab, setResultTab] = useState<"results" | "messages" | "logs">("results");
  const [inspectorTab, setInspectorTab] = useState<"columns" | "indexes" | "constraints" | "properties" | "ddl">("columns");

  return {
    activeTabId,
    inspectorTab,
    resultTab,
    setActiveTabId,
    setInspectorTab,
    setResultTab,
    setTabs,
    tabs,
  };
}
