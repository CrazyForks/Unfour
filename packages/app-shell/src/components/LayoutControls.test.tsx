// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { LayoutControls } from "./LayoutControls";

afterEach(cleanup);

describe("LayoutControls", () => {
  it("renders compact shell layout toggles with pressed state", () => {
    render(
      <LayoutControls
        bottomPanelCollapsed={false}
        onToggleBottomPanel={vi.fn()}
        onToggleInspector={vi.fn()}
        onToggleSidebar={vi.fn()}
        rightInspectorCollapsed
        sidebarCollapsed
      />,
    );

    expect(screen.getByRole("button", { name: "Expand sidebar" })).toHaveAttribute(
      "aria-pressed",
      "false",
    );
    expect(screen.getByRole("button", { name: "Toggle bottom panel" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    expect(screen.getByRole("button", { name: "Toggle inspector" })).toHaveAttribute(
      "aria-pressed",
      "false",
    );
  });

  it("invokes each layout toggle handler", () => {
    const onToggleBottomPanel = vi.fn();
    const onToggleInspector = vi.fn();
    const onToggleSidebar = vi.fn();

    render(
      <LayoutControls
        bottomPanelCollapsed
        onToggleBottomPanel={onToggleBottomPanel}
        onToggleInspector={onToggleInspector}
        onToggleSidebar={onToggleSidebar}
        rightInspectorCollapsed
        sidebarCollapsed={false}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Collapse sidebar" }));
    fireEvent.click(screen.getByRole("button", { name: "Toggle bottom panel" }));
    fireEvent.click(screen.getByRole("button", { name: "Toggle inspector" }));

    expect(onToggleSidebar).toHaveBeenCalledTimes(1);
    expect(onToggleBottomPanel).toHaveBeenCalledTimes(1);
    expect(onToggleInspector).toHaveBeenCalledTimes(1);
  });
});
