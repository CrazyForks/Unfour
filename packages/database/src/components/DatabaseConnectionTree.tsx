import { Database, MoreHorizontal, Play, RefreshCw, Table2 } from "lucide-react";
import type { DatabaseConnection } from "@unfour/command-client";
import {
  ConnectionStatus,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  EmptyState,
  IconButton,
  StatusBadge,
  TreeView,
  type TreeViewItem,
} from "@unfour/ui";

export function DatabaseConnectionTree({
  connections,
  onNewQuery,
  onRefresh,
  onSelectConnection,
  selectedConnectionId,
}: {
  connections: DatabaseConnection[];
  onNewQuery?: () => void;
  onRefresh?: () => void;
  onSelectConnection: (connection: DatabaseConnection) => void;
  selectedConnectionId: string | null;
}) {
  if (!connections.length) {
    return <EmptyState className="min-h-[72px]">No database connections</EmptyState>;
  }

  const items: TreeViewItem[] = connections.map((connection) => ({
    actions: (
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <IconButton label={`Database actions for ${connection.name}`}>
            <MoreHorizontal size={13} />
          </IconButton>
        </DropdownMenuTrigger>
        <DropdownMenuContent>
          <DropdownMenuItem onSelect={() => onSelectConnection(connection)}>Connect</DropdownMenuItem>
          <DropdownMenuItem onSelect={onNewQuery}>New Query</DropdownMenuItem>
          <DropdownMenuItem onSelect={onRefresh}>Refresh</DropdownMenuItem>
          <DropdownMenuItem>Copy Name</DropdownMenuItem>
          <DropdownMenuItem>Edit Connection</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    ),
    children: [
      {
        children: [
          {
            children: [
              { id: `${connection.id}:tables`, icon: <Table2 size={13} />, label: "Tables" },
              { id: `${connection.id}:views`, label: "Views" },
              { id: `${connection.id}:functions`, label: "Functions" },
              { id: `${connection.id}:sequences`, label: "Sequences" },
            ],
            id: `${connection.id}:public`,
            label: "public",
          },
        ],
        id: `${connection.id}:schemas`,
        label: "Schemas",
      },
      { id: `${connection.id}:databases`, label: "Databases" },
      { id: `${connection.id}:users`, label: "Users" },
    ],
    icon: <Database size={13} />,
    id: connection.id,
    label: connection.name,
    meta:
      selectedConnectionId === connection.id ? (
        <ConnectionStatus connected />
      ) : (
        <StatusBadge tone="success">
          {connection.driver}
        </StatusBadge>
      ),
    title: connection.name,
  }));

  return (
    <TreeView
      defaultExpandedIds={selectedConnectionId ? [selectedConnectionId, `${selectedConnectionId}:schemas`, `${selectedConnectionId}:public`] : []}
      items={items}
      onSelect={(item) => {
        const connection = connections.find((candidate) => candidate.id === item.id);
        if (connection) {
          onSelectConnection(connection);
        }
      }}
      selectedId={selectedConnectionId}
    />
  );
}

export function DatabaseSidebarToolbar({
  onNewQuery,
  onRefresh,
}: {
  onNewQuery?: () => void;
  onRefresh?: () => void;
}) {
  return (
    <div className="flex items-center gap-1">
      <IconButton label="New database query" onClick={onNewQuery}>
        <Play size={13} />
      </IconButton>
      <IconButton label="Refresh database connections" onClick={onRefresh}>
        <RefreshCw size={13} />
      </IconButton>
    </div>
  );
}
