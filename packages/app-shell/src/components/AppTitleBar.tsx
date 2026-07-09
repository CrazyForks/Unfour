import { Settings } from "lucide-react";
import type { Workspace } from "@unfour/command-client";
import { useState } from "react";
import {
  GlobalToolbar,
  IconButton,
  useI18n,
} from "@unfour/ui";
import { SettingsDialog } from "./settings/SettingsDialog";
import { WindowControls } from "./WindowControls";
import { WorkspaceMenu } from "./WorkspaceMenu";

export function AppTitleBar({
  activeWorkspace,
  onActivateWorkspace,
  workspaces,
}: {
  activeWorkspace?: Workspace;
  onActivateWorkspace: (workspaceId: string) => void;
  workspaces: Workspace[];
}) {
  const { t } = useI18n();
  const [settingsOpen, setSettingsOpen] = useState(false);

  return (
    <>
      <GlobalToolbar
        left={
          <WorkspaceMenu
            activeWorkspace={activeWorkspace}
            onActivateWorkspace={onActivateWorkspace}
            workspaces={workspaces}
          />
        }
        right={
          <>
            <IconButton label={t("app.titlebar.settings")} onClick={() => setSettingsOpen(true)}>
              <Settings size={15} />
            </IconButton>
            <WindowControls />
          </>
        }
      />
      <SettingsDialog onOpenChange={setSettingsOpen} open={settingsOpen} />
    </>
  );
}
