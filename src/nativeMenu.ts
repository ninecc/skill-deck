import {
  CheckMenuItem,
  Menu,
  MenuItem,
  PredefinedMenuItem,
  Submenu,
} from "@tauri-apps/api/menu";
import type { Messages } from "./i18n";
import type { Availability, CommandId } from "./commands";

export type CommandStates = Record<CommandId, Availability>;

const accelerator: Partial<Record<CommandId, string>> = {
  "find-installed": "CmdOrCtrl+F",
  "find-install": "CmdOrCtrl+Shift+I",
  "refresh-inventory": "CmdOrCtrl+R",
  settings: "CmdOrCtrl+,",
};

export function commandLabels(copy: Messages, translationOn: boolean) {
  return {
    "find-installed": copy.findInstalled,
    "find-install": copy.findInstall,
    "refresh-inventory": copy.refreshInventory,
    settings: copy.settings,
    "update-all": copy.updateAll,
    "translate-skill": translationOn
      ? copy.hideTranslation
      : copy.showTranslation,
    "reveal-skill": copy.revealRoot,
    "update-skill": copy.update,
    "remove-skill": copy.remove,
  } satisfies Record<CommandId, string>;
}

export async function installNativeMenu(
  copy: Messages,
  states: CommandStates,
  translationOn: boolean,
  dispatch: (id: CommandId) => void,
) {
  if (!("__TAURI_INTERNALS__" in window)) return () => undefined;
  const labels = commandLabels(copy, translationOn);
  const items = new Map<CommandId, MenuItem | CheckMenuItem>();
  const item = async (id: CommandId) => {
    const options = {
      id,
      text: labels[id],
      enabled: states[id].enabled,
      accelerator: accelerator[id],
      action: () => dispatch(id),
    };
    const value =
      id === "translate-skill"
        ? await CheckMenuItem.new({ ...options, checked: translationOn })
        : await MenuItem.new(options);
    items.set(id, value);
    return value;
  };
  const separator = () => PredefinedMenuItem.new({ item: "Separator" });
  const app = await Submenu.new({
    text: "Skill Deck",
    items: [
      await PredefinedMenuItem.new({ item: { About: null } }),
      await separator(),
      await item("settings"),
      await separator(),
      await PredefinedMenuItem.new({ item: "Hide" }),
      await PredefinedMenuItem.new({ item: "HideOthers" }),
      await PredefinedMenuItem.new({ item: "ShowAll" }),
      await separator(),
      await PredefinedMenuItem.new({ item: "Quit" }),
    ],
  });
  const inventory = await Submenu.new({
    text: copy.inventoryMenu,
    items: [
      await item("find-installed"),
      await item("find-install"),
      await item("refresh-inventory"),
      await separator(),
      await item("update-all"),
    ],
  });
  const edit = await Submenu.new({
    text: copy.editMenu,
    items: await Promise.all(
      (["Undo", "Redo", "Cut", "Copy", "Paste", "SelectAll"] as const).map(
        (role) => PredefinedMenuItem.new({ item: role }),
      ),
    ),
  });
  const skill = await Submenu.new({
    text: copy.skillMenu,
    items: [
      await item("translate-skill"),
      await item("reveal-skill"),
      await item("update-skill"),
      await separator(),
      await item("remove-skill"),
    ],
  });
  const windowMenu = await Submenu.new({
    text: copy.windowMenu,
    items: await Promise.all(
      (["Minimize", "Maximize", "Fullscreen", "BringAllToFront"] as const).map(
        (role) => PredefinedMenuItem.new({ item: role }),
      ),
    ),
  });
  const menu = await Menu.new({
    items: [app, inventory, edit, skill, windowMenu],
  });
  await menu.setAsAppMenu();
  return () => menu.close();
}

export async function popupSkillMenu(
  copy: Messages,
  states: CommandStates,
  translationOn: boolean,
  dispatch: (id: CommandId) => void,
) {
  if (!("__TAURI_INTERNALS__" in window)) return;
  const labels = commandLabels(copy, translationOn);
  const ids: CommandId[] = [
    "translate-skill",
    "reveal-skill",
    "update-skill",
    "remove-skill",
  ];
  const menu = await Menu.new({
    items: await Promise.all(
      ids.map((id) =>
        id === "translate-skill"
          ? CheckMenuItem.new({
              id,
              text: labels[id],
              enabled: states[id].enabled,
              checked: translationOn,
              action: () => dispatch(id),
            })
          : MenuItem.new({
              id,
              text: labels[id],
              enabled: states[id].enabled,
              action: () => dispatch(id),
            }),
      ),
    ),
  });
  await menu.popup();
  await menu.close();
}
