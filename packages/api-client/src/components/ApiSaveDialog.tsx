import { useState } from "react";
import {
  Button,
  Dialog,
  DialogBody,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
} from "@unfour/ui";

export function ApiSaveDialog({
  defaultFolder,
  defaultName,
  onCancel,
  onSave,
  open,
  saving,
}: {
  defaultFolder: string;
  defaultName: string;
  onCancel: () => void;
  onSave: (identity: { folderPath: string; name: string }) => void;
  open: boolean;
  saving: boolean;
}) {
  const [name, setName] = useState(defaultName);
  const [folderPath, setFolderPath] = useState(defaultFolder);

  return (
    <Dialog onOpenChange={(next) => !next && onCancel()} open={open}>
      <DialogContent title="Save API request">
        <DialogHeader>
          <DialogTitle>Save request</DialogTitle>
        </DialogHeader>
        <DialogBody className="space-y-3">
          <DialogDescription>
            Choose a recognizable name and Collection folder.
          </DialogDescription>
          <label className="block space-y-1">
            <span className="text-[12px] font-medium">Name</span>
            <Input
              autoFocus
              onChange={(event) => setName(event.target.value)}
              value={name}
            />
          </label>
          <label className="block space-y-1">
            <span className="text-[12px] font-medium">Folder</span>
            <Input
              onChange={(event) => setFolderPath(event.target.value)}
              placeholder="Examples / Auth"
              value={folderPath}
            />
          </label>
        </DialogBody>
        <DialogFooter>
          <Button onClick={onCancel} type="button" variant="ghost">
            Cancel
          </Button>
          <Button
            disabled={saving || !name.trim()}
            onClick={() => onSave({ folderPath, name: name.trim() })}
            type="button"
          >
            {saving ? "Saving" : "Save"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
