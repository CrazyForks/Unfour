import Editor from "@monaco-editor/react";
import { Play, Square } from "lucide-react";
import type { DatabaseConnection } from "@unfour/command-client";
import { Button, EmptyState, IconButton, Select, Toolbar, ToolbarGroup } from "@unfour/ui";

export function SqlEditorTab({
  connections,
  executePending,
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
  onRun: () => void;
  onSelectConnection: (connectionId: string) => void;
  onSqlChange: (sql: string) => void;
  onStop: () => void;
  pendingConfirmation: boolean;
  selectedConnectionId: string | null;
  sql: string;
}) {
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <Toolbar className="h-8">
        <ToolbarGroup className="min-w-0 flex-1">
          <Select
            aria-label="SQL editor connection"
            className="max-w-[260px]"
            onChange={(event) => onSelectConnection(event.target.value)}
            options={connections.map((connection) => ({ label: connection.name, value: connection.id }))}
            value={selectedConnectionId ?? ""}
          >
            {!connections.length && <option value="">No connections</option>}
          </Select>
          <span className="hidden text-[12px] text-[var(--u-color-text-soft)] sm:inline">Database selector pending backend support</span>
        </ToolbarGroup>
        <ToolbarGroup>
          <Button disabled={!selectedConnectionId || executePending} onClick={onRun} size="sm" type="button">
            <Play size={13} />
            {pendingConfirmation ? "Confirm run" : "Run"}
          </Button>
          <IconButton disabled={!executePending} label="Stop SQL execution" onClick={onStop}>
            <Square size={13} />
          </IconButton>
        </ToolbarGroup>
      </Toolbar>
      {connections.length === 0 ? (
        <EmptyState className="m-2 min-h-0 flex-1">Create or select a database connection to start writing SQL.</EmptyState>
      ) : (
        <Editor
          defaultLanguage="sql"
          onChange={(value) => onSqlChange(value ?? "")}
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
