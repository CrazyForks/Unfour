import type { ApiRequestTab } from "../model/request-tabs";
import type { ApiSplitDirection, ResponseTab } from "../model/types";
import { ResponseTabs } from "./ResponseTabs";

export function ApiResponseViewer({
  layoutDirection,
  onLayoutDirectionChange,
  onResponseTabChange,
  tab,
}: {
  layoutDirection: ApiSplitDirection;
  onLayoutDirectionChange: (direction: ApiSplitDirection) => void;
  onResponseTabChange: (tab: ResponseTab) => void;
  tab: ApiRequestTab;
}) {
  return (
    <ResponseTabs
      layoutDirection={layoutDirection}
      onLayoutDirectionChange={onLayoutDirectionChange}
      onResponseTabChange={onResponseTabChange}
      tab={tab}
    />
  );
}
