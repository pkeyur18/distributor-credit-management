// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { invokeCommand } from "./client";

afterEach(() => {
  clearMocks();
});

describe("invokeCommand", () => {
  it("resolves with the backend's response on success", async () => {
    mockIPC((cmd) => {
      if (cmd === "get_settings") return { hierarchyDepth: 4 };
      throw new Error(`unexpected command ${cmd}`);
    });

    const result = await invokeCommand<{ hierarchyDepth: number }>("get_settings");
    expect(result).toEqual({ hierarchyDepth: 4 });
  });

  it("passes the payload through to the backend command", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("add_member");
      expect(payload).toEqual({ name: "Asha" });
      return { id: 100002 };
    });

    await invokeCommand("add_member", { name: "Asha" });
  });

  it("rethrows a backend failure as a typed AppErrorPresentation, not the raw error", async () => {
    mockIPC(() => {
      throw { kind: "database", message: "constraint failed" };
    });

    await expect(invokeCommand("record_entry")).rejects.toMatchObject({
      kind: "database",
      message: "Something went wrong saving that. Try again.",
    });
  });
});
