// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import ModalShell from "./ModalShell";

let host: HTMLDivElement;
let root: ReturnType<typeof createRoot>;

beforeEach(() => {
  host = document.createElement("div");
  document.body.append(host);
  root = createRoot(host);
  (
    globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT: boolean }
  ).IS_REACT_ACT_ENVIRONMENT = true;
});

afterEach(async () => {
  await act(async () => root.unmount());
  host.remove();
});

it("focuses, contains Tab, closes on Escape and restores focus", async () => {
  const trigger = document.createElement("button");
  document.body.append(trigger);
  trigger.focus();
  const close = vi.fn();
  await act(async () =>
    root.render(
      <ModalShell labelledBy="title" onClose={close} returnFocus={trigger}>
        <h2 id="title">Title</h2>
        <button>First</button>
        <button>Last</button>
      </ModalShell>,
    ),
  );
  const buttons = host.querySelectorAll("button");
  expect(document.activeElement).toBe(buttons[0]);
  buttons[1].focus();
  buttons[1].dispatchEvent(
    new KeyboardEvent("keydown", { key: "Tab", bubbles: true }),
  );
  expect(document.activeElement).toBe(buttons[0]);
  buttons[0].dispatchEvent(
    new KeyboardEvent("keydown", { key: "Escape", bubbles: true }),
  );
  expect(close).toHaveBeenCalledOnce();
  await act(async () => root.unmount());
  expect(document.activeElement).toBe(trigger);
  trigger.remove();
  root = createRoot(host);
});

it("restores a stable fallback when the trigger disappeared", async () => {
  const fallback = document.createElement("button");
  fallback.id = "fallback";
  document.body.append(fallback);
  const trigger = document.createElement("button");
  document.body.append(trigger);
  await act(async () =>
    root.render(
      <ModalShell
        labelledBy="title"
        onClose={() => undefined}
        returnFocus={trigger}
        fallbackFocus="#fallback"
      >
        <h2 id="title">Title</h2>
        <button>Close</button>
      </ModalShell>,
    ),
  );
  trigger.remove();
  await act(async () => root.unmount());
  expect(document.activeElement).toBe(fallback);
  fallback.remove();
  root = createRoot(host);
});
