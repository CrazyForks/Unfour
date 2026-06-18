import { Shield, ShieldAlert, ShieldCheck, ShieldX } from "lucide-react";
import type { SshHostFingerprintInfo } from "@unfour/command-client";
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
} from "@unfour/ui";

export function HostKeyTrustDialog({
  existingFingerprint,
  host,
  mismatchError,
  onConfirm,
  onOpenChange,
  open,
  pending,
  port,
}: {
  existingFingerprint: SshHostFingerprintInfo | null | undefined;
  host: string;
  mismatchError?: string | null;
  onConfirm: () => void;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  pending?: boolean;
  port: number;
}) {
  const isMismatch = Boolean(mismatchError);
  const isFirstTrust = !existingFingerprint && !isMismatch;

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent>
        <DialogHeader>
          <div className="flex items-center gap-2">
            {isMismatch ? (
              <ShieldX className="shrink-0 text-[var(--u-color-danger)]" size={20} />
            ) : isFirstTrust ? (
              <Shield className="shrink-0 text-[var(--u-color-warning,orange)]" size={20} />
            ) : (
              <ShieldCheck className="shrink-0 text-[var(--u-color-success,green)]" size={20} />
            )}
            <div className="min-w-0">
              <DialogTitle>
                {isMismatch
                  ? "Host Key Verification Failed"
                  : isFirstTrust
                    ? "Confirm New Host Trust"
                    : "Host Key Verified"}
              </DialogTitle>
              <DialogDescription>
                {host}:{port}
              </DialogDescription>
            </div>
          </div>
          <DialogXClose />
        </DialogHeader>
        <DialogBody className="space-y-3">
          {isMismatch ? (
            <>
              <div className="rounded border border-[var(--u-color-danger,red)] bg-[var(--u-color-danger,red)]/10 p-3">
                <div className="flex items-start gap-2">
                  <ShieldAlert
                    className="mt-0.5 shrink-0 text-[var(--u-color-danger)]"
                    size={16}
                  />
                  <div className="space-y-1 text-[13px]">
                    <p className="font-semibold">
                      The server&apos;s host key has changed.
                    </p>
                    <p className="text-[var(--u-color-text-soft)]">
                      This could indicate a man-in-the-middle attack. The
                      connection was refused. If you trust the new key, reset
                      the stored fingerprint in the connection settings before
                      reconnecting.
                    </p>
                  </div>
                </div>
              </div>
              {existingFingerprint && (
                <div className="space-y-1.5 rounded border border-[var(--u-color-border)] p-2">
                  <span className="text-[11px] font-semibold uppercase text-[var(--u-color-text-soft)]">
                    Previously Trusted Fingerprint
                  </span>
                  <code className="block break-all text-[12px] text-[var(--u-color-text)]">
                    {existingFingerprint.fingerprint}
                  </code>
                </div>
              )}
              <p className="text-[12px] text-[var(--u-color-text-soft)]">
                {mismatchError}
              </p>
            </>
          ) : isFirstTrust ? (
            <>
              <div className="rounded border border-[var(--u-color-warning,orange)]/30 bg-[var(--u-color-warning,orange)]/5 p-3">
                <div className="flex items-start gap-2">
                  <Shield
                    className="mt-0.5 shrink-0 text-[var(--u-color-warning,orange)]"
                    size={16}
                  />
                  <div className="space-y-1 text-[13px]">
                    <p className="font-semibold">
                      You have not connected to this host before.
                    </p>
                    <p className="text-[var(--u-color-text-soft)]">
                      When you connect, the server&apos;s host-key fingerprint
                      will be recorded and trusted for future connections. If the
                      fingerprint changes later, you will be warned.
                    </p>
                  </div>
                </div>
              </div>
              <p className="text-[12px] text-[var(--u-color-text-soft)]">
                Do you want to proceed and trust this host?
              </p>
            </>
          ) : (
            <>
              <div className="space-y-1.5 rounded border border-[var(--u-color-border)] p-2">
                <span className="text-[11px] font-semibold uppercase text-[var(--u-color-text-soft)]">
                  Trusted Host-Key Fingerprint
                </span>
                <code className="block break-all text-[12px] text-[var(--u-color-text)]">
                  {existingFingerprint?.fingerprint}
                </code>
                <span className="block text-[11px] text-[var(--u-color-text-soft)]">
                  Trusted since{" "}
                  {existingFingerprint
                    ? new Date(existingFingerprint.createdAt).toLocaleDateString()
                    : "unknown"}
                </span>
              </div>
            </>
          )}
        </DialogBody>
        <DialogFooter>
          <DialogClose asChild>
            <Button type="button" variant="outline">
              {isMismatch ? "Close" : "Cancel"}
            </Button>
          </DialogClose>
          {isMismatch ? null : (
            <DialogClose asChild>
              <Button disabled={pending} onClick={onConfirm} type="button">
                <ShieldCheck size={14} />
                {isFirstTrust ? "Trust & Connect" : "Connect"}
              </Button>
            </DialogClose>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
