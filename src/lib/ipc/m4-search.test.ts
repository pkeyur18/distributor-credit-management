// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { getMemberDetail, getDirectChildrenChart, getAncestorChain, exportMemberDetailPdf } from "./m4-search";

afterEach(() => {
  clearMocks();
});

describe("m4-search IPC wrappers", () => {
  it("getMemberDetail sends flat memberId/periodMonth, periodMonth undefined when omitted", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("get_member_detail");
      expect(payload).toEqual({ memberId: 284913, periodMonth: undefined });
      return {};
    });
    await getMemberDetail(284913);
  });

  it("getMemberDetail passes periodMonth through when given", async () => {
    mockIPC((_cmd, payload) => {
      expect(payload).toEqual({ memberId: 284913, periodMonth: "2026-05" });
      return {};
    });
    await getMemberDetail(284913, "2026-05");
  });

  it("getDirectChildrenChart spreads the request flat, no `input` wrapper", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("get_direct_children_chart");
      expect(payload).toEqual({ memberId: 100001, fullTree: true, periodMonth: "2026-06" });
      return { nodes: [], slabTable: [] };
    });
    await getDirectChildrenChart({ memberId: 100001, fullTree: true, periodMonth: "2026-06" });
  });

  it("getAncestorChain sends a flat scalar `memberId`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("get_ancestor_chain");
      expect(payload).toEqual({ memberId: 284913 });
      return { chain: [] };
    });
    await getAncestorChain(284913);
  });

  it("exportMemberDetailPdf sends flat memberId/periodMonth/outputPath", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("export_member_detail_pdf");
      expect(payload).toEqual({
        memberId: 284913,
        periodMonth: "2026-06",
        outputPath: "/tmp/out.pdf",
      });
      return { filePath: "/tmp/out.pdf" };
    });
    await exportMemberDetailPdf(284913, "2026-06", "/tmp/out.pdf");
  });
});
