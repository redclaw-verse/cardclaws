import { MAX_HISTORY, newLayerId, useCardStore } from "../../src/stores/cardStore";
import { CardDefinition, DEFAULT_SETTINGS, emptySide, TextLayer } from "../../src/types/card";

function baseCard(): CardDefinition {
  return {
    id: "card-1",
    ownerId: "user-1",
    handle: "omar",
    version: 1,
    face: emptySide(),
    back: emptySide(),
    palette: { colors: [] },
    settings: DEFAULT_SETTINGS,
  };
}

function textLayer(text: string): TextLayer {
  return {
    id: newLayerId(),
    type: "text",
    x: 0.1,
    y: 0.1,
    width: 0.8,
    height: 0.1,
    opacity: 1,
    zIndex: 1,
    text,
    fontFamily: "System",
    fontWeight: 700,
    fontSize: 28,
    lineHeight: 32,
    letterSpacing: 0,
    color: "#ffffff",
    align: "left",
  };
}

beforeEach(() => {
  useCardStore.getState().load(baseCard());
});

describe("cardStore undo/redo", () => {
  it("starts with no history", () => {
    const s = useCardStore.getState();
    expect(s.canUndo()).toBe(false);
    expect(s.canRedo()).toBe(false);
    expect(s.card?.face.layers).toHaveLength(0);
  });

  it("adds a layer and can undo/redo it", () => {
    const s = useCardStore.getState();
    s.addLayer("face", textLayer("Omar"));
    expect(useCardStore.getState().card?.face.layers).toHaveLength(1);
    expect(useCardStore.getState().canUndo()).toBe(true);

    useCardStore.getState().undo();
    expect(useCardStore.getState().card?.face.layers).toHaveLength(0);
    expect(useCardStore.getState().canRedo()).toBe(true);

    useCardStore.getState().redo();
    expect(useCardStore.getState().card?.face.layers).toHaveLength(1);
  });

  it("a new edit clears the redo stack", () => {
    const s = useCardStore.getState();
    s.addLayer("face", textLayer("A"));
    useCardStore.getState().undo();
    expect(useCardStore.getState().canRedo()).toBe(true);

    useCardStore.getState().addLayer("face", textLayer("B"));
    expect(useCardStore.getState().canRedo()).toBe(false);
    expect(useCardStore.getState().card?.face.layers[0]).toMatchObject({ text: "B" });
  });

  it("updateLayer is reversible", () => {
    const layer = textLayer("Before");
    useCardStore.getState().addLayer("face", layer);
    useCardStore.getState().updateLayer("face", layer.id, { text: "After" } as Partial<TextLayer>);

    const after = useCardStore.getState().card?.face.layers[0] as TextLayer;
    expect(after.text).toBe("After");

    useCardStore.getState().undo();
    const before = useCardStore.getState().card?.face.layers[0] as TextLayer;
    expect(before.text).toBe("Before");
  });

  it("caps history at MAX_HISTORY and can still undo that many times", () => {
    for (let i = 0; i < MAX_HISTORY + 10; i++) {
      useCardStore.getState().addLayer("face", textLayer(`L${i}`));
    }
    expect(useCardStore.getState().past.length).toBe(MAX_HISTORY);

    let undos = 0;
    while (useCardStore.getState().canUndo()) {
      useCardStore.getState().undo();
      undos++;
    }
    expect(undos).toBe(MAX_HISTORY);
  });

  it("reorderLayer swaps adjacent z-indexes and is reversible", () => {
    const a = { ...textLayer("A"), zIndex: 1 };
    const b = { ...textLayer("B"), zIndex: 2 };
    useCardStore.getState().addLayer("face", a);
    useCardStore.getState().addLayer("face", b);

    // Move A up: A and B swap z-indexes → A above B.
    useCardStore.getState().reorderLayer("face", a.id, "up");
    const za = () =>
      useCardStore.getState().card!.face.layers.find((l) => l.id === a.id)!.zIndex;
    const zb = () =>
      useCardStore.getState().card!.face.layers.find((l) => l.id === b.id)!.zIndex;
    expect(za()).toBe(2);
    expect(zb()).toBe(1);

    useCardStore.getState().undo();
    expect(za()).toBe(1);
    expect(zb()).toBe(2);
  });

  it("reorderLayer past an edge is a no-op", () => {
    const a = { ...textLayer("A"), zIndex: 1 };
    useCardStore.getState().addLayer("face", a);
    const before = useCardStore.getState().past.length;
    useCardStore.getState().reorderLayer("face", a.id, "down"); // already bottom
    // The mutation still snapshots, but ordering is unchanged.
    expect(useCardStore.getState().card!.face.layers[0].zIndex).toBe(1);
    expect(useCardStore.getState().past.length).toBe(before + 1);
  });

  it("edits profile bio and links with undo support", () => {
    const s = useCardStore.getState();
    s.setBio("Founder & CEO");
    expect(useCardStore.getState().card?.profile?.bio).toBe("Founder & CEO");

    useCardStore.getState().addLink({
      id: "l1",
      type: "linkedin",
      label: "LinkedIn",
      url: "https://linkedin.com/in/omar",
      iconSlug: "linkedin",
    });
    expect(useCardStore.getState().card?.profile?.links).toHaveLength(1);

    useCardStore.getState().updateLink("l1", { label: "My LinkedIn" });
    expect(useCardStore.getState().card?.profile?.links[0].label).toBe("My LinkedIn");

    useCardStore.getState().removeLink("l1");
    expect(useCardStore.getState().card?.profile?.links).toHaveLength(0);

    // Undo the removal, then the label edit.
    useCardStore.getState().undo();
    expect(useCardStore.getState().card?.profile?.links).toHaveLength(1);
    useCardStore.getState().undo();
    expect(useCardStore.getState().card?.profile?.links[0].label).toBe("LinkedIn");
  });

  it("undo/redo are no-ops at the ends", () => {
    expect(() => useCardStore.getState().undo()).not.toThrow();
    expect(() => useCardStore.getState().redo()).not.toThrow();
    expect(useCardStore.getState().card?.face.layers).toHaveLength(0);
  });
});
