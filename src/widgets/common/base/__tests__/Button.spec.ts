import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import Button from "../Button.vue";

describe("Button", () => {
  it("emits click when enabled", async () => {
    const wrapper = mount(Button, {
      slots: {
        default: "保存",
      },
    });

    await wrapper.get("button").trigger("click");

    expect(wrapper.emitted("click")).toHaveLength(1);
  });

  it("does not emit click while loading", async () => {
    const wrapper = mount(Button, {
      props: {
        loading: true,
      },
      slots: {
        default: "保存",
      },
    });

    await wrapper.get("button").trigger("click");

    expect(wrapper.emitted("click")).toBeUndefined();
    expect(wrapper.get("button").attributes("disabled")).toBeDefined();
  });
});
