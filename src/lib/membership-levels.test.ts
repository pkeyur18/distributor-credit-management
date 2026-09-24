import { describe, expect, it } from "vitest";
import { MEMBERSHIP_LEVEL_NAMES, membershipLevelName } from "./membership-levels";

describe("membershipLevelName", () => {
  it("maps rank 1..4 to the draft names and anything else to an em dash", () => {
    expect(MEMBERSHIP_LEVEL_NAMES).toEqual(["Gold", "Platinum", "Diamond", "Ace"]);
    expect(membershipLevelName(0)).toBe("—");
    expect(membershipLevelName(1)).toBe("Gold");
    expect(membershipLevelName(4)).toBe("Ace");
    expect(membershipLevelName(5)).toBe("—");
  });
});
