import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import Input from "../Input.vue";

describe("Input", () => {
  it("emits v-model updates from native input", async () => {
    const wrapper = mount(Input, {
      props: {
        modelValue: "",
        label: "姓名",
      },
    });

    await wrapper.get("input").setValue("张三");

    expect(wrapper.emitted("update:modelValue")).toEqual([[ "张三" ]]);
  });

  it("renders help text when there is no error", () => {
    const wrapper = mount(Input, {
      props: {
        helpText: "请输入教师姓名",
      },
    });

    expect(wrapper.text()).toContain("请输入教师姓名");
  });
});
