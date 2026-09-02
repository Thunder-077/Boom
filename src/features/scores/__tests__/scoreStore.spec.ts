import { describe, expect, it } from "vitest";
import { createScoreStore } from "../store";

describe("score store", () => {
  it("initializes grade options before the first async load", () => {
    const store = createScoreStore();

    expect(store.viewState.gradeOptions).toEqual([]);
  });
});
