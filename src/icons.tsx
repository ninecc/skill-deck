import type { ComponentType, SVGProps } from "react";
import ChevronDown from "~icons/lucide/chevron-down";
import FileText from "~icons/lucide/file-text";
import Folder from "~icons/lucide/folder";
import FolderOpen from "~icons/lucide/folder-open";
import Image from "~icons/lucide/image";
import Languages from "~icons/lucide/languages";
import PackagePlus from "~icons/lucide/package-plus";
import RefreshCw from "~icons/lucide/refresh-cw";
import Search from "~icons/lucide/search";
import Settings from "~icons/lucide/sliders-horizontal";
import Trash from "~icons/lucide/trash-2";
import UpdateAll from "~icons/lucide/arrow-down-to-line";
import UpdateSkill from "~icons/lucide/download";
import X from "~icons/lucide/x";

export type IconName =
  | "search"
  | "install"
  | "settings"
  | "refresh"
  | "update-all"
  | "update-skill"
  | "folder"
  | "folder-open"
  | "file"
  | "image"
  | "translate"
  | "trash"
  | "download"
  | "close"
  | "chevron";

type SvgIcon = ComponentType<SVGProps<SVGSVGElement>>;

const icons = {
  search: Search,
  install: PackagePlus,
  settings: Settings,
  refresh: RefreshCw,
  "update-all": UpdateAll,
  "update-skill": UpdateSkill,
  folder: Folder,
  "folder-open": FolderOpen,
  file: FileText,
  image: Image,
  translate: Languages,
  trash: Trash,
  download: UpdateSkill,
  close: X,
  chevron: ChevronDown,
} satisfies Record<IconName, SvgIcon>;

export function Icon({ name }: { name: IconName }) {
  const Glyph = icons[name];
  return (
    <Glyph
      className="icon"
      data-icon={name}
      aria-hidden="true"
      focusable="false"
    />
  );
}
