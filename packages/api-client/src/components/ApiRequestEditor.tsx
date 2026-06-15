import type { KeyValue } from "@unfour/command-client";
import type { RequestParamsTab } from "../model/types";
import { RequestParamsTabs } from "./RequestParamsTabs";

export function ApiRequestEditor({
  body,
  envVariables,
  headers,
  onBodyChange,
  onEnvVariablesChange,
  onHeadersChange,
  onQueryChange,
  onSaveEnvironment,
  onTabChange,
  query,
  savingEnvironment,
  tab,
}: {
  body: string;
  envVariables: KeyValue[];
  headers: KeyValue[];
  onBodyChange: (value: string) => void;
  onEnvVariablesChange: (items: KeyValue[]) => void;
  onHeadersChange: (items: KeyValue[]) => void;
  onQueryChange: (items: KeyValue[]) => void;
  onSaveEnvironment: () => void;
  onTabChange: (tab: RequestParamsTab) => void;
  query: KeyValue[];
  savingEnvironment: boolean;
  tab: RequestParamsTab;
}) {
  return (
    <section className="flex min-h-0 min-w-0 flex-1 flex-col">
      <RequestParamsTabs
        body={body}
        envVariables={envVariables}
        headers={headers}
        onBodyChange={onBodyChange}
        onEnvVariablesChange={onEnvVariablesChange}
        onHeadersChange={onHeadersChange}
        onQueryChange={onQueryChange}
        onSaveEnvironment={onSaveEnvironment}
        onTabChange={onTabChange}
        query={query}
        savingEnvironment={savingEnvironment}
        tab={tab}
      />
    </section>
  );
}
