import { isValidHandle, validateHandle } from "../../src/utils/handleValidation";

describe("handleValidation", () => {
  it("accepts valid handles", () => {
    for (const h of ["omar", "red-claw", "card123", "a1b"]) {
      expect(isValidHandle(h)).toBe(true);
    }
  });

  it("rejects too short / too long", () => {
    expect(validateHandle("ab")).toBe("too_short");
    expect(validateHandle("a".repeat(31))).toBe("too_long");
  });

  it("rejects leading/trailing hyphen", () => {
    expect(validateHandle("-omar")).toBe("leading_or_trailing_hyphen");
    expect(validateHandle("omar-")).toBe("leading_or_trailing_hyphen");
  });

  it("rejects invalid characters", () => {
    expect(validateHandle("Omar")).toBe("invalid_chars");
    expect(validateHandle("om ar")).toBe("invalid_chars");
    expect(validateHandle("omar!")).toBe("invalid_chars");
  });

  it("rejects reserved handles", () => {
    expect(validateHandle("admin")).toBe("reserved");
    expect(validateHandle("api")).toBe("reserved");
  });
});
