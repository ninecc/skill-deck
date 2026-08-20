import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { Icon, type IconName } from "./icons";

const names: IconName[] = [
  "search",
  "install",
  "settings",
  "refresh",
  "update-all",
  "update-skill",
  "folder",
  "folder-open",
  "file",
  "image",
  "translate",
  "trash",
  "download",
  "close",
  "chevron",
  "runtime-warning",
  "empty-inventory",
  "preview-placeholder",
  "preview-warning",
];

describe("offline icon adapter", () => {
  it.each(names)("renders %s as a static decorative Lucide SVG", (name) => {
    const markup = renderToStaticMarkup(<Icon name={name} />);
    expect(markup).toContain("<svg");
    expect(markup).toContain(`data-icon="${name}"`);
    expect(markup).toContain('aria-hidden="true"');
    expect(markup).not.toContain("api.iconify.design");
  });
});
