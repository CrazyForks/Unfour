import { describe, expect, it } from "vitest";
import {
  activeWorkspaceEnvironmentName,
  defaultTaskRunInputs,
  mergeActiveWorkspaceVariables,
} from "./task-run-inputs";

describe("task run workspace variable defaults", () => {
  it("merges workspace vars then overlays the active environment", () => {
    const merged = mergeActiveWorkspaceVariables(
      [
        {
          key: "source_image",
          value: "workspace-image",
          isEnabled: true,
          isSecret: false,
        },
        {
          key: "archive_name",
          value: "from-workspace",
          isEnabled: true,
          isSecret: false,
        },
        {
          key: "disabled_key",
          value: "nope",
          isEnabled: false,
          isSecret: false,
        },
      ],
      [
        {
          name: "Dev",
          isActive: true,
          variables: [
            {
              key: "archive_name",
              value: "from-env",
              isEnabled: true,
              isSecret: false,
            },
            {
              key: "token",
              value: "secret-token",
              isEnabled: true,
              isSecret: true,
            },
          ],
        },
        {
          name: "Prod",
          isActive: false,
          variables: [
            {
              key: "archive_name",
              value: "prod-should-not-win",
              isEnabled: true,
              isSecret: false,
            },
          ],
        },
      ],
    );

    expect(merged.get("source_image")?.value).toBe("workspace-image");
    expect(merged.get("archive_name")?.value).toBe("from-env");
    expect(merged.get("token")).toEqual({
      key: "token",
      value: "secret-token",
      isSecret: true,
    });
    expect(merged.has("disabled_key")).toBe(false);
  });

  it("prefills matching placeholders case-insensitively and leaves the rest empty", () => {
    const variables = mergeActiveWorkspaceVariables(
      [
        {
          key: "SOURCE_IMAGE",
          value: "nginx:latest",
          isEnabled: true,
          isSecret: false,
        },
        {
          key: "API_TOKEN",
          value: "secret",
          isEnabled: true,
          isSecret: true,
        },
      ],
      [],
    );

    expect(
      defaultTaskRunInputs(
        ["source_image", "target_image", "API_TOKEN"],
        variables,
      ),
    ).toEqual({
      inputs: {
        source_image: "nginx:latest",
        target_image: "",
        API_TOKEN: "secret",
      },
      secretNames: ["API_TOKEN"],
      filledFromWorkspace: ["source_image", "API_TOKEN"],
    });
  });

  it("reports the active environment name when present", () => {
    expect(
      activeWorkspaceEnvironmentName([
        { name: " Dev ", isActive: true, variables: [] },
        { name: "Prod", isActive: false, variables: [] },
      ]),
    ).toBe("Dev");
    expect(activeWorkspaceEnvironmentName([{ isActive: false, variables: [] }])).toBe(
      null,
    );
  });
});
