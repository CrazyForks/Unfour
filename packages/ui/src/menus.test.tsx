// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
} from "./menus";
import { DropdownMenu, DropdownMenuTrigger } from "./menus-primitives";

afterEach(cleanup);

describe("ContextMenu", () => {
  it("moves focus into menu items with keyboard navigation", () => {
    const onSelect = vi.fn();

    render(
      <ContextMenu>
        <ContextMenuTrigger asChild>
          <button type="button">Request tab</button>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem onSelect={onSelect}>Open</ContextMenuItem>
          <ContextMenuItem>Duplicate</ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>,
    );

    fireEvent.contextMenu(screen.getByRole("button", { name: "Request tab" }), {
      clientX: 24,
      clientY: 24,
    });

    const menu = screen.getByRole("menu");
    const openItem = screen.getByRole("menuitem", { name: "Open" });

    fireEvent.keyDown(menu, { key: "ArrowDown" });

    expect(openItem).toHaveFocus();
  });
});

describe("DropdownMenu", () => {
  it("keeps menu item weight normal when opened from a bold tree row", async () => {
    render(
      <div className="font-semibold">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <button type="button">More</button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem>Open</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>,
    );

    fireEvent.pointerDown(screen.getByRole("button", { name: "More" }));

    expect(await screen.findByRole("menu")).toHaveClass("font-normal");
  });
});
