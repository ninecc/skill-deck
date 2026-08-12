import { invoke } from "@tauri-apps/api/core";

export interface RuntimeStatus {
  ready: boolean;
  version: string | null;
  nodeVersion: string | null;
  message: string | null;
}

export interface InstalledSkill {
  name: string;
  path: string;
  scope: string;
  agents: string[];
  source: string | null;
  sourceUrl: string | null;
  sourceType: string | null;
}

export interface SearchResult {
  name: string;
  slug: string;
  source: string;
  installs: number;
}

export interface InstallSettings {
  agents: string[];
  copy: boolean;
}

export interface CommandResult {
  inventory: InstalledSkill[];
  changedSkills: string[];
  targetObserved: boolean | null;
  diagnostics: string;
}

export type ViewerKind = "markdown" | "text" | "code" | "image" | "unsupported";

export interface FileEntry {
  path: string;
  name: string;
  level: number;
  directory: boolean;
  size: number;
  viewer: ViewerKind;
  unsupportedReason: string | null;
}

export interface FileContent {
  path: string;
  viewer: ViewerKind;
  size: number;
  text: string | null;
  dataUrl: string | null;
  translatable: boolean;
  unsupportedReason?: string;
}

export interface TranslationResult {
  translatedText: string;
  detectedSourceLanguage: string | null;
}

export const runtimeStatus = () => invoke<RuntimeStatus>("runtime_status");
export const retryRuntime = () => invoke<RuntimeStatus>("retry_runtime");
export const listSkills = () => invoke<InstalledSkill[]>("list_skills");
export const searchSkills = (query: string) =>
  invoke<SearchResult[]>("search_skills", { query });
export const addSkill = (
  source: string,
  skill: string | null,
  settings: InstallSettings,
) => invoke<CommandResult>("add_skill", { source, skill, settings });
export const removeSkill = (name: string) =>
  invoke<CommandResult>("remove_skill", { name });
export const updateSkill = (name: string | null) =>
  invoke<CommandResult>("update_skill", { name });
export const previewTree = (skill: string) =>
  invoke<FileEntry[]>("preview_tree", { skill });
export const readPreview = (skill: string, path: string) =>
  invoke<FileContent>("read_preview", { skill, path });
export const revealPath = (skill: string, path: string | null) =>
  invoke<void>("reveal_path", { skill, path });
export const translatePreview = (
  skill: string,
  path: string,
  targetLanguage: string,
) =>
  invoke<TranslationResult>("translate_preview", {
    skill,
    path,
    targetLanguage,
  });

interface ErrorPayload {
  code?: unknown;
  message?: unknown;
  diagnostics?: unknown;
}

function payload(value: unknown): ErrorPayload | null {
  return typeof value === "object" && value !== null ? value : null;
}

export function commandErrorCode(value: unknown): string | null {
  const code = payload(value)?.code;
  return typeof code === "string" ? code : null;
}

export function commandErrorMessage(value: unknown): string {
  if (value instanceof Error) return value.message;
  const data = payload(value);
  const message = typeof data?.message === "string" ? data.message : null;
  const diagnostics =
    typeof data?.diagnostics === "string" && data.diagnostics
      ? data.diagnostics
      : null;
  return [message, diagnostics].filter(Boolean).join("\n") || "Unknown error";
}
