import {
  buildPersona,
  EMPTY_PROFILE,
  isProfileComplete,
  Profile,
} from "../../src/stores/profileLogic";

const base: Profile = { ...EMPTY_PROFILE };

describe("profileLogic", () => {
  test("isProfileComplete requires a non-empty name", () => {
    expect(isProfileComplete(base)).toBe(false);
    expect(isProfileComplete({ ...base, displayName: "  " })).toBe(false);
    expect(isProfileComplete({ ...base, displayName: "Omar" })).toBe(true);
  });

  test("buildPersona returns undefined for an empty profile", () => {
    expect(buildPersona(base)).toBeUndefined();
  });

  test("buildPersona maps fields and joins vibes, dropping empties", () => {
    const persona = buildPersona({
      ...base,
      title: "Founder",
      vibes: ["Cinematic", "Luxe"],
      brandColors: ["#ff3b30"],
      goal: "networking",
    });
    expect(persona).toEqual({
      role: "Founder",
      vibe: "Cinematic, Luxe",
      colors: ["#ff3b30"],
      goal: "networking",
    });
  });
});
