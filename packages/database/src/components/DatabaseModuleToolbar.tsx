import { Eraser, Play, Plug, RefreshCw, Square, Unplug } from "lucide-react";
import type { DatabaseConnection } from "@unfour/command-client";
import {
  Button,
  ConnectionStatus,
  IconButton,
  Select,
  Toolbar,
  ToolbarGroup,
  useI18n,
} from "@unfour/ui";
import type { DatabaseConnectionStatus } from "../model/types";

export function DatabaseModuleToolbar({
  connectionStatus,
  connections,
  canRunSql,
  executePending,
  onClearSql,
  onConnect,
  onDisconnect,
  onNewQuery,
  onRefresh,
  onRun,
  onSelectConnection,
  onStop,
  pendingConfirmation,
  selectedConnectionId,
  sqlDirty,
}: {
  connectionStatus: DatabaseConnectionStatus;
  connections: DatabaseConnection[];
  canRunSql: boolean;
  executePending: boolean;
  onClearSql: () => void;
  onConnect: () => void;
  onDisconnect: () => void;
  onNewQuery: () => void;
  onRefresh: () => void;
  onRun: () => void;
  onSelectConnection: (connectionId: string) => void;
  onStop: () => void;
  pendingConfirmation: boolean;
  selectedConnectionId: string | null;
  sqlDirty: boolean;
}) {
  const { t } = useI18n();
  const selected = connections.find((connection) => connection.id === selectedConnectionId);
  const connected = connectionStatus === "connected" || connectionStatus === "connecting";
  const connectionStatusLabel =
    connectionStatus === "connecting"
      ? t("common.actions.connecting")
      : t(`database.connection.${connectionStatus}`);

  return (
    <Toolbar>
      <ToolbarGroup>
        <Button
          onClick={() => onNewQuery()}
          size="sm"
          type="button"
          variant="outline"
        >
          {t("database.actions.newQuery")}
        </Button>
        <Button disabled={!canRunSql || executePending} onClick={onRun} size="sm" type="button">
          <Play size={14} />
          {pendingConfirmation ? t("database.actions.confirmRun") : t("database.actions.run")}
        </Button>
        <IconButton disabled={!executePending} label={t("database.actions.stopSql")} onClick={onStop}>
          <Square size={14} />
        </IconButton>
        <IconButton disabled={!sqlDirty || executePending} label={t("database.actions.clearSql")} onClick={onClearSql}>
          <Eraser size={14} />
        </IconButton>
        <IconButton label={t("database.actions.refreshModule")} onClick={onRefresh}>
          <RefreshCw size={14} />
        </IconButton>
      </ToolbarGroup>
      <ToolbarGroup className="max-w-[560px]">
        <ConnectionStatus
          label={connectionStatusLabel}
          status={connectionStatus === "failed" ? "error" : connectionStatus}
          variant="dot"
        />
        <Select
          aria-label={t("database.connection.selectAria")}
          className="w-[220px]"
          onChange={(event) => onSelectConnection(event.target.value)}
          options={connections.map((connection) => ({
            label: connection.name,
            value: connection.id,
          }))}
          value={selectedConnectionId ?? ""}
        >
          {!selectedConnectionId && <option value="">{t("database.connection.select")}</option>}
          {!connections.length && <option value="">{t("database.connection.none")}</option>}
        </Select>
        {connected ? (
          <Button disabled={!selected || executePending} onClick={onDisconnect} size="sm" type="button" variant="outline">
            <Unplug size={13} />
            {t("common.actions.disconnect")}
          </Button>
        ) : (
          <Button disabled={!selected || executePending} onClick={onConnect} size="sm" type="button" variant="outline">
            <Plug size={13} />
            {t("common.actions.connect")}
          </Button>
        )}
      </ToolbarGroup>
    </Toolbar>
  );
}
