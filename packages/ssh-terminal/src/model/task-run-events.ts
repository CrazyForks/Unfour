import type { SshTaskRunEvent } from "@unfour/command-client";

const MAX_TASK_RUN_EVENTS = 5_000;

/**
 * Append live task events while keeping transfer progress bounded. A transfer
 * can emit many updates for the same step, but the UI only needs the newest
 * snapshot to render its progress bar and transcript summary.
 */
export function appendTaskRunEvents(
  currentEvents: SshTaskRunEvent[],
  nextEvents: SshTaskRunEvent[],
) {
  const retainedNewestFirst: SshTaskRunEvent[] = [];
  const seenTransferSteps = new Set<string>();
  const merged = [...currentEvents, ...nextEvents];

  for (let index = merged.length - 1; index >= 0; index -= 1) {
    const event = merged[index]!;
    if (event.kind === "transfer" && event.stepId) {
      if (seenTransferSteps.has(event.stepId)) continue;
      seenTransferSteps.add(event.stepId);
    }
    retainedNewestFirst.push(event);
  }

  return retainedNewestFirst.reverse().slice(-MAX_TASK_RUN_EVENTS);
}
