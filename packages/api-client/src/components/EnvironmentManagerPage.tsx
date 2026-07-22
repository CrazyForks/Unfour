import { useEffect, useMemo, useState } from "react";
import type {
  WorkspaceEnvironment,
  WorkspaceVariable,
  WorkspaceVariableInput,
} from "@unfour/command-client";
import { WORKSPACE_VARIABLES_SELECTION_ID } from "./environment-manager-selection";
import { Button, Input, VariableTable, useI18n } from "@unfour/ui";
import { useApiEnvironments } from "../hooks/useApiEnvironments";
import { useWorkspaceVariables } from "../hooks/useWorkspaceVariables";
import { formatError } from "../model/api-request-state";
import { findDuplicateEnvironmentName, nextEnvironmentName } from "../request-utils";

export type EnvironmentManagerInitialMode =
  | { kind: "manage"; nonce: number }
  | { kind: "new"; nonce: number }
  | { kind: "workspace"; nonce: number }
  | { environmentId: string; kind: "edit"; nonce: number };

type ExistingEnvironmentDraft = {
  id: string;
  kind: "existing";
  name: string;
  sourceName: string;
  sourceUpdatedAt: string;
  sourceVariables: WorkspaceVariableInput[];
  variables: WorkspaceVariableInput[];
};

type WorkspaceVariablesDraft = {
  kind: "workspace";
  sourceVariables: WorkspaceVariableInput[];
  variables: WorkspaceVariableInput[];
};

type EnvironmentDraft =
  | { kind: "none" }
  | { kind: "new"; name: string; variables: WorkspaceVariableInput[] }
  | WorkspaceVariablesDraft
  | ExistingEnvironmentDraft;

export function EnvironmentManagerPage({
  initialMode,
  onDirtyChange,
  onSelectionChange,
  workspaceId,
}: {
  initialMode: EnvironmentManagerInitialMode;
  onDirtyChange?: (dirty: boolean) => void;
  onSelectionChange?: (selectionId: string | null) => void;
  workspaceId: string;
}) {
  const { t } = useI18n();
  const { createMut, environments, isLoading, updateMut } =
    useApiEnvironments(workspaceId);
  const {
    isLoading: workspaceVariablesLoading,
    replaceMut,
    variables: workspaceVariables,
  } = useWorkspaceVariables(workspaceId);
  const [draft, setDraft] = useState<EnvironmentDraft>({ kind: "none" });
  const [saveError, setSaveError] = useState<string | null>(null);

  const dirty = isDraftDirty(draft);
  const selectedEnvironment =
    draft.kind === "existing"
      ? environments.find((environment) => environment.id === draft.id) ?? null
      : null;
  const duplicateName =
    draft.kind === "new" || draft.kind === "existing"
      ? findDuplicateEnvironmentName(
          environments,
          draft.name,
          draft.kind === "existing" ? draft.id : undefined,
        )
      : null;
  const draftName =
    draft.kind === "new" || draft.kind === "existing" ? draft.name : "";
  const saving = createMut.isPending || updateMut.isPending || replaceMut.isPending;
  const persistedVariables =
    draft.kind === "none" ? [] : persistableVariables(draft.variables);
  const hasIncompleteVariable = persistedVariables.some((variable) => !variable.key.trim());
  const hasDuplicateVariable = hasDuplicateKeys(persistedVariables);
  const overridingKeys = useMemo(
    () => new Set(workspaceVariables.map((variable) => variable.key)),
    [workspaceVariables],
  );

  useEffect(() => {
    onDirtyChange?.(dirty);
  }, [dirty, onDirtyChange]);

  useEffect(() => {
    onSelectionChange?.(
      draft.kind === "workspace"
        ? WORKSPACE_VARIABLES_SELECTION_ID
        : draft.kind === "existing"
          ? draft.id
          : null,
    );
  }, [draft, onSelectionChange]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- external open intent resets the editor draft
    setSaveError(null);
    if (initialMode.kind === "new") {
      setDraft({
        kind: "new",
        name: nextEnvironmentName(t("api.environment.defaultName"), environments),
        variables: [],
      });
      return;
    }
    if (initialMode.kind === "workspace") {
      setDraft(draftFromWorkspaceVariables(workspaceVariables));
      return;
    }
    if (initialMode.kind === "edit") {
      const target = environments.find(
        (environment) => environment.id === initialMode.environmentId,
      );
      setDraft(target ? draftFromEnvironment(target) : { kind: "none" });
      return;
    }
    const target =
      environments.find((environment) => environment.isActive) ?? environments[0] ?? null;
    setDraft(
      target
        ? draftFromEnvironment(target)
        : isLoading
          ? { kind: "none" }
          : draftFromWorkspaceVariables(workspaceVariables),
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps -- nonce is the explicit open-intent boundary
  }, [initialMode.nonce]);

  useEffect(() => {
    if (dirty) return;
    if (draft.kind === "none") {
      const target =
        initialMode.kind === "edit"
          ? environments.find(
              (environment) => environment.id === initialMode.environmentId,
            )
          : environments.find((environment) => environment.isActive) ??
            environments[0];
      if (target) {
        // eslint-disable-next-line react-hooks/set-state-in-effect -- hydrate an async environment open intent
        setDraft(draftFromEnvironment(target));
      } else if (
        initialMode.kind !== "edit" &&
        !isLoading &&
        !workspaceVariablesLoading
      ) {
        setDraft(draftFromWorkspaceVariables(workspaceVariables));
      }
      return;
    }
    if (draft.kind === "workspace") {
      const next = draftFromWorkspaceVariables(workspaceVariables);
      if (JSON.stringify(next.sourceVariables) !== JSON.stringify(draft.sourceVariables)) {
        setDraft(next);
      }
      return;
    }
    if (draft.kind === "existing") {
      if (!selectedEnvironment) {
        const fallback = environments[0];
        setDraft(
          fallback
            ? draftFromEnvironment(fallback)
            : draftFromWorkspaceVariables(workspaceVariables),
        );
      } else if (selectedEnvironment.updatedAt !== draft.sourceUpdatedAt) {
        setDraft(draftFromEnvironment(selectedEnvironment));
      }
    }
  }, [
    dirty,
    draft,
    environments,
    initialMode,
    isLoading,
    selectedEnvironment,
    workspaceVariables,
    workspaceVariablesLoading,
  ]);

  const saveDisabled =
    draft.kind === "none" ||
    saving ||
    !dirty ||
    hasIncompleteVariable ||
    hasDuplicateVariable ||
    ((draft.kind === "new" || draft.kind === "existing") &&
      (!draft.name.trim() || Boolean(duplicateName)));

  async function saveDraft() {
    if (draft.kind === "none") return;
    const variables = persistableVariables(draft.variables);
    setSaveError(null);
    try {
      if (draft.kind === "workspace") {
        const saved = await replaceMut.mutateAsync(variables);
        setDraft(draftFromWorkspaceVariables(saved));
        return;
      }
      const name = draft.name.trim();
      if (draft.kind === "new") {
        const created = await createMut.mutateAsync(name);
        const saved = variables.length
          ? await updateMut.mutateAsync({ id: created.id, name, variables })
          : created;
        setDraft(draftFromEnvironment(saved));
        return;
      }
      const saved = await updateMut.mutateAsync({ id: draft.id, name, variables });
      setDraft(draftFromEnvironment(saved));
    } catch (error) {
      setSaveError(formatError(error));
    }
  }

  if (draft.kind === "none") {
    return (
      <div className="flex h-full items-center justify-center p-6 text-[12px] text-[var(--u-color-text-muted)]">
        {isLoading || workspaceVariablesLoading
          ? t("common.state.loading")
          : t("api.environment.selectHint")}
      </div>
    );
  }

  const isWorkspace = draft.kind === "workspace";
  return (
    <main className="h-full min-w-0 flex-1 overflow-y-auto bg-[var(--u-color-bg)]">
      <div className="mx-auto flex min-h-full max-w-5xl flex-col gap-3 p-4">
        <div className="flex shrink-0 items-start justify-between gap-3 border-b border-[var(--u-color-border)] pb-3">
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <h2 className="truncate text-[14px] font-semibold text-[var(--u-color-text)]">
                {isWorkspace
                  ? t("variables.workspaceVariables")
                  : draft.kind === "new"
                    ? t("api.environment.newEnvironment")
                    : draft.sourceName}
              </h2>
              {selectedEnvironment?.isActive && (
                <span className="inline-flex h-5 items-center gap-1.5 rounded-[var(--u-radius-sm)] px-1.5 text-[11px] font-medium text-[var(--u-color-primary)]">
                  <span className="h-1.5 w-1.5 rounded-full bg-current" />
                  {t("api.environment.activeBadge")}
                </span>
              )}
              {dirty && (
                <span className="text-[11px] text-[var(--u-color-text-soft)]">
                  {t("api.environment.unsaved")}
                </span>
              )}
            </div>
            <p className="mt-1 text-[12px] text-[var(--u-color-text-muted)]">
              {isWorkspace
                ? t("variables.workspaceDescription")
                : t("api.environment.workspaceVariables", {
                    count: persistedVariables.length,
                  })}
            </p>
          </div>
          <Button disabled={saveDisabled} onClick={() => void saveDraft()} size="sm" type="button">
            {saving ? t("api.actions.saving") : t("api.environment.save")}
          </Button>
        </div>

        {!isWorkspace && (
          <label className="grid gap-1 text-[12px] text-[var(--u-color-text-muted)]">
            {t("api.environment.nameLabel")}
            <Input
              aria-invalid={duplicateName ? true : undefined}
              onChange={(event) =>
                setDraft((current) =>
                  current.kind === "new" || current.kind === "existing"
                    ? { ...current, name: event.target.value }
                    : current,
                )
              }
              value={draft.kind === "new" || draft.kind === "existing" ? draft.name : ""}
            />
          </label>
        )}
        {duplicateName && (
          <div className="text-[12px] text-[var(--u-color-danger)]">
            {t("api.environment.duplicateName", { name: draftName.trim() })}
          </div>
        )}
        <VariableTable
          items={draft.variables}
          onChange={(variables) =>
            setDraft((current) =>
              current.kind === "none" ? current : { ...current, variables },
            )
          }
          overridingKeys={isWorkspace ? undefined : overridingKeys}
          title={t("api.environment.variablesLabel")}
        />
        {saveError && (
          <div className="text-[12px] text-[var(--u-color-danger)]">{saveError}</div>
        )}
      </div>
    </main>
  );
}

function draftFromEnvironment(environment: WorkspaceEnvironment): ExistingEnvironmentDraft {
  const variables = environment.variables.map(toInput);
  return {
    id: environment.id,
    kind: "existing",
    name: environment.name,
    sourceName: environment.name,
    sourceUpdatedAt: environment.updatedAt,
    sourceVariables: variables,
    variables,
  };
}

function draftFromWorkspaceVariables(variables: WorkspaceVariable[]): WorkspaceVariablesDraft {
  const inputs = variables.map(toInput);
  return { kind: "workspace", sourceVariables: inputs, variables: inputs };
}

function toInput(variable: WorkspaceVariable): WorkspaceVariableInput {
  return {
    id: variable.id,
    key: variable.key,
    value: variable.value,
    isSecret: variable.isSecret,
    isEnabled: variable.isEnabled,
    description: variable.description,
    sortOrder: variable.sortOrder,
  };
}

function isDraftDirty(draft: EnvironmentDraft) {
  if (draft.kind === "none") return false;
  if (draft.kind === "new") {
    return Boolean(draft.name.trim()) || persistableVariables(draft.variables).length > 0;
  }
  return (
    (draft.kind === "existing" && draft.name !== draft.sourceName) ||
    JSON.stringify(draft.variables) !== JSON.stringify(draft.sourceVariables)
  );
}

function persistableVariables(variables: WorkspaceVariableInput[]) {
  return variables
    .filter(
      (variable) =>
        variable.key.trim() ||
        variable.value ||
        variable.description?.trim() ||
        variable.isSecret ||
        !variable.isEnabled,
    )
    .map((variable, index) => ({ ...variable, sortOrder: index }));
}

function hasDuplicateKeys(variables: WorkspaceVariableInput[]) {
  const keys = new Set<string>();
  for (const variable of variables) {
    const key = variable.key.trim().toLowerCase();
    if (key && keys.has(key)) return true;
    keys.add(key);
  }
  return false;
}
