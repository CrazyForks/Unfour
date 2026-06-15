import { Columns2, Rows2 } from "lucide-react";
import { IconButton } from "@unfour/ui";
import type { ApiSplitDirection } from "../model/types";

export function ApiWorkspaceLayoutToggle({
  direction,
  onChange,
}: {
  direction: ApiSplitDirection;
  onChange: (direction: ApiSplitDirection) => void;
}) {
  const next = direction === "vertical" ? "horizontal" : "vertical";
  return (
    <IconButton
      label={`Switch to ${next === "vertical" ? "top and bottom" : "side by side"} layout`}
      onClick={() => onChange(next)}
      tooltip={`Switch to ${next === "vertical" ? "top and bottom" : "side by side"} layout`}
    >
      {direction === "vertical" ? <Columns2 size={14} /> : <Rows2 size={14} />}
    </IconButton>
  );
}
