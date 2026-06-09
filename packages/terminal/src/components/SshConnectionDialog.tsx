import type { FormEvent, ReactNode } from "react";
import { Save, Trash2 } from "lucide-react";
import type { SshConnectionInput } from "@unfour/command-client";
import {
  Button,
  Dialog,
  DialogBody,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogXClose,
  ErrorState,
  Input,
  Select,
} from "@unfour/ui";
import { CredentialReferenceControl } from "./CredentialReferenceControl";
import { HostKeyFingerprint } from "./HostKeyFingerprint";
import { formatTerminalError } from "../model/errors";

export function SshConnectionDialog({
  canDelete,
  error,
  form,
  onDelete,
  onOpenChange,
  onSubmit,
  onUpdate,
  open,
  pending,
  workspaceId,
}: {
  canDelete: boolean;
  error?: unknown;
  form: SshConnectionInput;
  onDelete: () => void;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent) => void;
  onUpdate: (patch: Partial<SshConnectionInput>) => void;
  open: boolean;
  pending?: boolean;
  workspaceId: string;
}) {
  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent>
        <form className="flex min-h-0 flex-col" onSubmit={onSubmit}>
          <DialogHeader>
            <div className="min-w-0">
              <DialogTitle>{form.id ? "Edit SSH Connection" : "New SSH Connection"}</DialogTitle>
              <DialogDescription>
                Connection metadata is saved here; secrets stay behind credential references.
              </DialogDescription>
            </div>
            <DialogXClose />
          </DialogHeader>
          <DialogBody className="space-y-3">
            <FieldGroup title="Name">
              <Input
                autoFocus
                onChange={(event) => onUpdate({ name: event.target.value })}
                value={form.name}
              />
            </FieldGroup>
            <div className="grid grid-cols-[minmax(0,1fr)_84px] gap-2">
              <FieldGroup title="Host">
                <Input onChange={(event) => onUpdate({ host: event.target.value })} value={form.host} />
              </FieldGroup>
              <FieldGroup title="Port">
                <Input
                  onChange={(event) =>
                    onUpdate({ port: event.target.value ? Number(event.target.value) : null })
                  }
                  type="number"
                  value={form.port ?? ""}
                />
              </FieldGroup>
            </div>
            <FieldGroup title="Username">
              <Input
                onChange={(event) => onUpdate({ username: event.target.value })}
                value={form.username}
              />
            </FieldGroup>
            <FieldGroup title="Authentication Type">
              <Select
                onChange={(event) =>
                  onUpdate({
                    authKind: event.target.value as SshConnectionInput["authKind"],
                    keyPath: event.target.value === "private-key" ? form.keyPath : null,
                  })
                }
                options={[
                  { label: "Password", value: "password" },
                  { label: "Private key", value: "private-key" },
                ]}
                value={form.authKind}
              />
            </FieldGroup>
            {form.authKind === "private-key" && (
              <FieldGroup title="Private Key Path">
                <Input
                  onChange={(event) => onUpdate({ keyPath: event.target.value })}
                  placeholder="C:\\Users\\me\\.ssh\\id_ed25519"
                  value={form.keyPath ?? ""}
                />
              </FieldGroup>
            )}
            <CredentialReferenceControl
              kind={form.authKind === "private-key" ? "ssh-key-passphrase" : "ssh-password"}
              label={`${form.name || "SSH"} credential`}
              onChange={(credentialRef) => onUpdate({ credentialRef })}
              value={form.credentialRef}
              workspaceId={workspaceId}
            />
            {Boolean(form.host) && (
              <HostKeyFingerprint host={form.host} port={form.port ?? 22} />
            )}
            {Boolean(error) && (
              <ErrorState className="min-h-0 justify-start py-2 text-left">
                {formatTerminalError(error)}
              </ErrorState>
            )}
          </DialogBody>
          <DialogFooter>
            <Button
              disabled={!canDelete || pending}
              onClick={onDelete}
              type="button"
              variant="ghost"
            >
              <Trash2 size={14} />
              Delete
            </Button>
            <DialogClose asChild>
              <Button type="button" variant="outline">
                Cancel
              </Button>
            </DialogClose>
            <Button disabled={pending} type="submit">
              <Save size={14} />
              Save
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

function FieldGroup({ children, title }: { children: ReactNode; title: string }) {
  return (
    <label className="block space-y-1.5">
      <span className="text-[12px] font-semibold uppercase text-[var(--u-color-text-soft)]">
        {title}
      </span>
      {children}
    </label>
  );
}
