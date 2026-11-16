import { afterEach, expect, it } from "vitest";
import styles from "../../../app/react-shell.css?raw";

let fixture: HTMLDivElement;
let stylesheet: HTMLStyleElement;

afterEach(() => {
  fixture?.remove();
  stylesheet?.remove();
});

it("preserves native time input sizing and centers both time columns", () => {
  stylesheet = document.createElement("style");
  stylesheet.textContent = styles;
  document.head.append(stylesheet);
  fixture = document.createElement("div");
  fixture.innerHTML = `<table class="exam-table"><thead><tr>
    <th>科目</th><th>考试日期</th><th>开始时间</th><th>结束时间</th>
    </tr></thead><tbody><tr><td>语文</td><td>04-28</td>
    <td><input class="time-input" type="time" value="08:00"></td>
    <td><input class="time-input" type="time" value="14:10"></td>
    </tr></tbody></table>`;
  document.body.append(fixture);

  // 保护原生控件的尺寸策略；实际 AM/PM 排版仍需在桌面 WebView 中验证。
  const inputRules = Array.from(stylesheet.sheet!.cssRules)
    .filter((rule): rule is CSSStyleRule => "selectorText" in rule && rule.selectorText === ".time-input");
  expect(inputRules.map((rule) => rule.style.width).filter(Boolean).pop()).toBe("auto");
  expect(inputRules.map((rule) => rule.style.minWidth).filter(Boolean).pop()).toBe("max-content");
  for (const cell of fixture.querySelectorAll("th:nth-child(3), th:nth-child(4), td:nth-child(3), td:nth-child(4)")) {
    expect(getComputedStyle(cell).textAlign).toBe("center");
  }
});
