import { describe, expect, it, vi } from "vitest";
import { registerScopedListener } from "../asyncListener";

describe("registerScopedListener", () => {
  it("ignores events after disposal and unregisters even when registration resolves late", async () => {
    let resolve!: (stop: () => void) => void;
    let emit!: (event: string) => void;
    const stop = vi.fn();
    const handler = vi.fn();
    const dispose = registerScopedListener<string>((callback) => {
      emit = callback;
      return new Promise((done) => { resolve = done; });
    }, handler);

    emit("active");
    dispose();
    emit("stale");
    resolve(stop);
    await Promise.resolve();
    expect(handler).toHaveBeenCalledOnce();
    expect(handler).toHaveBeenCalledWith("active");
    expect(stop).toHaveBeenCalledOnce();
  });

  it("unregisters an already registered listener on disposal", async () => {
    const stop = vi.fn();
    const dispose = registerScopedListener<string>(async () => stop, vi.fn());
    await Promise.resolve();
    dispose();
    expect(stop).toHaveBeenCalledOnce();
  });
});
