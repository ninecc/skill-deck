export const commandIds = [
  "find-installed",
  "find-install",
  "refresh-inventory",
  "settings",
  "update-all",
  "translate-skill",
  "reveal-skill",
  "update-skill",
  "remove-skill",
] as const;

export type CommandId = (typeof commandIds)[number];
export type UnavailableReason =
  | "runtime-unavailable"
  | "inventory-empty"
  | "no-skill-selected"
  | "mutation-active"
  | "modal-active"
  | "document-loading"
  | "unsupported-document";

export type Availability =
  | { enabled: true; reason: null }
  | { enabled: false; reason: UnavailableReason };

export interface CommandContext {
  runtimeReady: boolean;
  inventoryCount: number;
  selected: boolean;
  mutationActive: boolean;
  modal: "settings" | "discovery" | "remove" | null;
  document: "loading" | "supported" | "unsupported" | "none";
}

const enabled: Availability = { enabled: true, reason: null };
const disabled = (reason: UnavailableReason): Availability => ({
  enabled: false,
  reason,
});

export function commandAvailability(
  id: CommandId,
  context: CommandContext,
): Availability {
  if (id === "settings")
    return context.modal ? disabled("modal-active") : enabled;
  if (id === "find-installed")
    return context.modal ? disabled("modal-active") : enabled;
  if (id === "find-install")
    return context.modal
      ? disabled("modal-active")
      : context.mutationActive
        ? disabled("mutation-active")
        : enabled;
  if (!context.runtimeReady) return disabled("runtime-unavailable");
  if (
    context.mutationActive &&
    (id === "refresh-inventory" ||
      id === "update-all" ||
      id === "update-skill" ||
      id === "remove-skill")
  )
    return disabled("mutation-active");
  if (id === "refresh-inventory") return enabled;
  if (id === "update-all")
    return context.inventoryCount ? enabled : disabled("inventory-empty");
  if (!context.selected) return disabled("no-skill-selected");
  if (id === "remove-skill" && context.modal) return disabled("modal-active");
  if (id === "translate-skill") {
    if (context.document === "loading") return disabled("document-loading");
    if (context.document !== "supported")
      return disabled("unsupported-document");
  }
  return enabled;
}

export function createDispatcher(
  context: () => CommandContext,
  execute: (id: CommandId) => void,
) {
  return (id: CommandId) => {
    const availability = commandAvailability(id, context());
    if (availability.enabled) execute(id);
    return availability;
  };
}

export function shortcutCommand(
  event: Pick<KeyboardEvent, "key" | "metaKey" | "ctrlKey" | "shiftKey">,
): CommandId | null {
  if (!event.metaKey && !event.ctrlKey) return null;
  const key = event.key.toLowerCase();
  if (key === "f" && !event.shiftKey) return "find-installed";
  if (key === "i" && event.shiftKey) return "find-install";
  if (key === ",") return "settings";
  if (key === "r" && !event.shiftKey) return "refresh-inventory";
  return null;
}

export function isNarrowBackShortcut(
  event: Pick<KeyboardEvent, "key" | "metaKey" | "altKey">,
  narrow: boolean,
) {
  return (
    narrow &&
    ((event.altKey && event.key === "ArrowLeft") ||
      (event.metaKey && event.key === "["))
  );
}
