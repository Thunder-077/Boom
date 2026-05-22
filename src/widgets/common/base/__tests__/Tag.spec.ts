import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import Tag from "../Tag.vue";

describe("Tag", () => {
  it("renders slot content and variant class", () => {
    const wrapper = mount(Tag, {
      props: {
        variant: "success",
        size: "sm",
      },
      slots: {
        default: "已完成",
      },
    });

    expect(wrapper.text()).toContain("已完成");
    expect(wrapper.classes()).toContain("tag-success");
    expect(wrapper.classes()).toContain("tag-sm");
  });

  it("emits click from keyboard when clickable", async () => {
    const wrapper = mount(Tag, {
      props: {
        clickable: true,
      },
    });

    await wrapper.trigger("keydown.enter");

    expect(wrapper.emitted("click")).toHaveLength(1);
  });
});
