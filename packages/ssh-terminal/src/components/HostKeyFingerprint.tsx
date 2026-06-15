import { useEffect, useState } from "react";
import { Shield, ShieldAlert, Trash2 } from "lucide-react";
import type { SshHostFingerprintInfo } from "@unfour/command-client";
import { getSshHostFingerprint, resetSshHostFingerprint } from "@unfour/command-client";
import { Button } from "@unfour/ui";

export function HostKeyFingerprint({
  host,
  port,
}: {
  host: string;
  port: number;
}) {
  const [info, setInfo] = useState<SshHostFingerprintInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const trimmedHost = host.trim();
  const validPort = port > 0;

  if (!trimmedHost || !validPort) {
    if (info !== null) setInfo(null);
    if (error !== null) setError(null);
  }

  useEffect(() => {
    if (!trimmedHost || !validPort) {
      return;
    }
    let cancelled = false;
    setLoading(true);
    setError(null);
    getSshHostFingerprint({ host: trimmedHost, port })
      .then((result) => {
        if (!cancelled) setInfo(result);
      })
      .catch((err) => {
        if (!cancelled) setError(String(err?.message ?? err));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [trimmedHost, port, validPort]);

  function handleReset() {
    if (!trimmedHost || !validPort) return;
    setLoading(true);
    setError(null);
    resetSshHostFingerprint({ host: trimmedHost, port })
      .then(() => {
        setInfo(null);
      })
      .catch((err) => {
        setError(String(err?.message ?? err));
      })
      .finally(() => {
        setLoading(false);
      });
  }

  if (!trimmedHost || !validPort) return null;

  if (loading && !info) {
    return (
      <div className="flex items-center gap-2 text-[12px] text-[var(--u-color-text-soft)]">
        <Shield size={14} />
        <span>Loading host-key fingerprint...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center gap-2 text-[12px] text-[var(--u-color-danger)]">
        <ShieldAlert size={14} />
        <span>{error}</span>
      </div>
    );
  }

  if (!info) {
    return (
      <div className="flex items-center gap-2 text-[12px] text-[var(--u-color-text-soft)]">
        <Shield size={14} />
        <span>No trusted fingerprint recorded yet. It will be saved on first connection.</span>
      </div>
    );
  }

  return (
    <div className="space-y-1.5 rounded border border-[var(--u-color-border)] p-2">
      <div className="flex items-center justify-between">
        <span className="text-[12px] font-semibold uppercase text-[var(--u-color-text-soft)]">
          Trusted Host-Key Fingerprint
        </span>
        <Button
          disabled={loading}
          onClick={handleReset}
          size="sm"
          type="button"
          variant="ghost"
        >
          <Trash2 size={12} />
          Reset
        </Button>
      </div>
      <code className="block break-all text-[12px] text-[var(--u-color-text)]">
        {info.fingerprint}
      </code>
      <span className="block text-[11px] text-[var(--u-color-text-soft)]">
        Trusted since {new Date(info.createdAt).toLocaleDateString()}
      </span>
    </div>
  );
}
