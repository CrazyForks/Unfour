import { MoreHorizontal, Play, Plug, RefreshCw, Square, WandSparkles } from "lucide-react";
import type { DatabaseConnection } from "@unfour/command-client";
import {
  Button,
  ConnectionStatus,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  Select,
  Toolbar,
  ToolbarGroup,
} from "@unfour/ui";

export function DatabaseModuleToolbar({
  connections,
  executePending,
  onNewQuery,
  onRefresh,
  onRun,
  onSelectConnection,
  onStop,
  pendingConfirmation,
  selectedConnectionId,
}: {
  connections: DatabaseConnection[];
  executePending: boolean;
  onNewQuery: () => void;
  onRefresh: () => void;
  onRun: () => void;
  onSelectConnection: (connectionId: string) => void;
  onStop: () => void;
  pendingConfirmation: boolean;
  selectedConnectionId: string | null;
}) {
  const selected = connections.find((connection) => connection.id === selectedConnectionId);
  return (
    <Toolbar>
      <ToolbarGroup>
        <Button onClick={onNewQuery} size="sm" type="button" variant="outline">
          New Query
        </Button>
        <Button disabled={!selectedConnectionId || executePending} onClick={onRun} size="sm" type="button">
          <Play size={14} />
          {pendingConfirmation ? "Confirm run" : "Run"}
        </Button>
        <IconButton disabled={!executePending} label="Stop SQL execution" onClick={onStop}>
          <Square size={14} />
        </IconButton>
        <IconButton label="Refresh database module" onClick={onRefresh}>
          <RefreshCw size={14} />
        </IconButton>
      </ToolbarGroup>
      <ToolbarGroup className="max-w-[520px]">
        <ConnectionStatus connected={Boolean(selected)} label={selected ? selected.name : "disconnected"} />
        <Select
          aria-label="Database connection"
          className="w-[220px]"
          onChange={(event) => onSelectConnection(event.target.value)}
          options={connections.map((connection) => ({
            label: connection.name,
            value: connection.id,
          }))}
          value={selectedConnectionId ?? ""}
        >
          {!connections.length && <option value="">No connections</option>}
        </Select>
        <IconButton disabled={!selected} label="Connect database">
          <Plug size={14} />
        </IconButton>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <IconButton label="More database actions">
              <MoreHorizontal size={14} />
            </IconButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem>
              <WandSparkles size={13} />
              Format SQL
            </DropdownMenuItem>
            <DropdownMenuItem>Explain Query</DropdownMenuItem>
            <DropdownMenuItem>Export Result</DropdownMenuItem>
            <DropdownMenuItem>Import SQL</DropdownMenuItem>
            <DropdownMenuItem>Duplicate Tab</DropdownMenuItem>
            <DropdownMenuItem>Close Other Tabs</DropdownMenuItem>
            <DropdownMenuItem>Connection Settings</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </ToolbarGroup>
    </Toolbar>
  );
}
