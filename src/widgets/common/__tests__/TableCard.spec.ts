import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import TableCard from "../TableCard.vue";

describe("TableCard", () => {
  it("renders title and slot content", () => {
    const wrapper = mount(TableCard, {
      props: {
        title: "教师列表",
        meta: "共 10 位",
      },
      slots: {
        default: "<div>table-body</div>",
      },
    });

    expect(wrapper.text()).toContain("教师列表");
    expect(wrapper.text()).toContain("共 10 位");
    expect(wrapper.text()).toContain("table-body");
  });
});
