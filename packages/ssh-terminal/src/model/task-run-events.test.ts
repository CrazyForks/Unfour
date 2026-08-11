import { describe, expect, it } from "vitest";
import type { SshTaskRunEvent } from "@unfour/command-client";
import { appendTaskRunEvents } from "./task-run-events";

function event(partial: Partial<SshTaskRunEvent>): SshTaskRunEvent {
  return {
    runId: "run-1",
    taskId: "task-1",
    kind: "output",
    stepId: null,
    stepName: null,
    stepType: null,
    position: null,
    status: null,
    stream: null,
    data: null,
    exitCode: null,
    durationMs: null,
    direction: null,
    transferredBytes: null,
    totalBytes: null,
    bytesPerSecond: null,
    error: null,
    createdAt: "2026-01-01T00:00:00.000Z",
    ...partial,
  };
}

describe("appendTaskRunEvents", () => {
  it("keeps only the latest transfer progress for each step", () => {
    const retained = appendTaskRunEvents(
      [
        event({ kind: "step", stepId: "download", status: "running" }),
        event({
          kind: "transfer",
          stepId: "download",
          transferredBytes: 256,
          totalBytes: 1_024,
        }),
      ],
      [
        event({
          kind: "transfer",
          stepId: "download",
          transferredBytes: 768,
          totalBytes: 1_024,
        }),
        event({
          kind: "transfer",
          stepId: "download",
          transferredBytes: 1_024,
          totalBytes: 1_024,
        }),
        event({ kind: "step", stepId: "download", status: "success" }),
      ],
    );

    expect(retained.map((item) => item.kind)).toEqual(["step", "transfer", "step"]);
    expect(retained[1]?.transferredBytes).toBe(1_024);
  });

  it("bounds retained non-progress events", () => {
    const retained = appendTaskRunEvents(
      [],
      Array.from({ length: 5_100 }, (_, index) =>
        event({ data: `${index}\n`, createdAt: `event-${index}` }),
      ),
    );

    expect(retained).toHaveLength(5_000);
    expect(retained[0]?.data).toBe("100\n");
    expect(retained.at(-1)?.data).toBe("5099\n");
  });
});
