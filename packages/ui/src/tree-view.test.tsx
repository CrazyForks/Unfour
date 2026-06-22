// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { TreeView } from "./tree-view";

afterEach(cleanup);

describe("TreeView", () => {
  it("uses a compact disclosure control in sidebar rows", () => {
    render(
      <TreeView
        items={[
          {
            id: "parent",
            label: "Parent",
            children: [{ id: "child", label: "Child" }],
          },
        ]}
      />,
    );

    expect(screen.getByRole("button", { name: "Expand" })).toHaveClass("w-4");
  });
});
