import Editor, { type OnMount } from "@monaco-editor/react";
import { Eraser, Play, Square } from "lucide-react";
import { useEffect, useRef } from "react";
import type { DatabaseConnection } from "@unfour/command-client";
import { Button, EmptyState, IconButton, Select, Toolbar, ToolbarGroup, useI18n } from "@unfour/ui";

export function SqlEditorTab({
  connections,
  executePending,
  onClearSql,
  onRun,
  onSelectConnection,
  onSqlChange,
  onStop,
  pendingConfirmation,
  selectedConnectionId,
  sql,
}: {
  connections: DatabaseConnection[];
  executePending: boolean;
  onClearSql: () => void;
  onRun: () => void;
  onSelectConnection: (connectionId: string) => void;
  onSqlChange: (sql: string) => void;
  onStop: () => void;
  pendingConfirmation: boolean;
  selectedConnectionId: string | null;
  sql: string;
}) {
  const { t } = useI18n();
  const onRunRef = useRef(onRun);
  const selectedConnection = connections.find((connection) => connection.id === selectedConnectionId);

  useEffect(() => {
    onRunRef.current = onRun;
  }, [onRun]);

  const handleMount: OnMount = (editor, monaco) => {
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => onRunRef.current());
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <Toolbar className="h-8">
        <ToolbarGroup className="min-w-0 flex-1">
          <Select
            aria-label={t("database.editor.connectionAria")}
            className="max-w-[260px]"
            onChange={(event) => onSelectConnection(event.target.value)}
            options={connections.map((connection) => ({ label: connection.name, value: connection.id }))}
            value={selectedConnectionId ?? ""}
          >
            {!selectedConnectionId && <option value="">{t("database.connection.select")}</option>}
            {!connections.length && <option value="">{t("database.connection.none")}</option>}
          </Select>
          <span className="hidden min-w-0 truncate text-[12px] text-[var(--u-color-text-soft)] md:inline">
            {selectedConnection ? connectionContext(selectedConnection) : t("database.editor.noConnectionSelected")}
          </span>
        </ToolbarGroup>
        <ToolbarGroup>
          <IconButton disabled={!sql.trim() || executePending} label={t("database.actions.clearSql")} onClick={onClearSql}>
            <Eraser size={13} />
          </IconButton>
          <Button disabled={!selectedConnectionId || executePending} onClick={onRun} size="sm" type="button">
            <Play size={13} />
            {pendingConfirmation ? t("database.actions.confirmRun") : t("database.actions.run")}
          </Button>
          <IconButton disabled={!executePending} label={t("database.actions.stopSql")} onClick={onStop}>
            <Square size={13} />
          </IconButton>
        </ToolbarGroup>
      </Toolbar>
      {connections.length === 0 ? (
        <EmptyState className="m-2 min-h-0 flex-1">{t("database.editor.empty")}</EmptyState>
      ) : (
        <Editor
          defaultLanguage="sql"
          onChange={(value) => onSqlChange(value ?? "")}
          onMount={handleMount}
          options={{
            fontSize: 13,
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            wordWrap: "on",
          }}
          value={sql}
        />
      )}
    </div>
  );
}

function connectionContext(connection: DatabaseConnection) {
  const database = connection.database ?? connection.sqlitePath ?? "default";
  return `${connection.driver} / ${database}`;
}
