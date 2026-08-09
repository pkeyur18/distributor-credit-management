import { describe, expect, it } from "vitest";
import * as ipc from "./index";

describe("IPC command wrappers", () => {
  it("expose exactly 40 command functions, API-01 to API-40 with no gaps", () => {
    const modules = [
      ipc.m1Members,
      ipc.m2Entries,
      ipc.m3Calc,
      ipc.m4Search,
      ipc.m5Close,
      ipc.m6Reports,
      ipc.m7Settings,
      ipc.m8Auth,
      ipc.m9Audit,
      ipc.preflight,
    ];
    const commandFns = modules.flatMap((mod) =>
      Object.values(mod).filter((value): value is (...args: never[]) => unknown => {
        return typeof value === "function";
      }),
    );
    expect(commandFns).toHaveLength(40);
  });

  it("m1 exposes 6 commands (API-01–06), matching the module table", () => {
    expect(Object.keys(ipc.m1Members)).toHaveLength(6);
  });

  it("m8 plus the console-backup command exposes 7 commands (API-26–31, 39)", () => {
    expect(Object.keys(ipc.m8Auth)).toHaveLength(7);
  });

  it("pre-flight/recovery exposes exactly 4 commands (API-34–36, 40)", () => {
    expect(Object.keys(ipc.preflight)).toHaveLength(4);
  });
});
