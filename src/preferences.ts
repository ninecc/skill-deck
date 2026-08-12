export type Theme = "system" | "light" | "dark" | "sand" | "plum";

export const themes: Theme[] = ["system", "light", "dark", "sand", "plum"];
export const languages = [
  ["en", "English"],
  ["zh-Hans", "简体中文"],
  ["zh-Hant", "繁體中文"],
  ["ja", "日本語"],
  ["ko", "한국어"],
  ["es", "Español"],
  ["fr", "Français"],
  ["de", "Deutsch"],
  ["pt", "Português"],
  ["it", "Italiano"],
  ["ru", "Русский"],
  ["ar", "العربية"],
  ["hi", "हिन्दी"],
] as const;
export type TargetLanguage = (typeof languages)[number][0];

export const agentOptions = [
  "aider-desk",
  "amp",
  "replit",
  "universal",
  "antigravity",
  "antigravity-cli",
  "astrbot",
  "autohand-code",
  "augment",
  "bob",
  "claude-code",
  "openclaw",
  "cline",
  "dexto",
  "kimi-code-cli",
  "loaf",
  "warp",
  "zed",
  "codearts-agent",
  "codebuddy",
  "codemaker",
  "codestudio",
  "codex",
  "command-code",
  "continue",
  "cortex",
  "crush",
  "cursor",
  "deepagents",
  "devin",
  "droid",
  "firebender",
  "forgecode",
  "gemini-cli",
  "github-copilot",
  "goose",
  "grok",
  "hermes-agent",
  "inference-sh",
  "jazz",
  "junie",
  "iflow-cli",
  "kilo",
  "kimchi",
  "kiro-cli",
  "kode",
  "lingma",
  "mcpjam",
  "minimax-code",
  "mistral-vibe",
  "moxby",
  "mux",
  "opencode",
  "openhands",
  "ona",
  "pi",
  "qoder",
  "qoder-cn",
  "qwen-code",
  "reasonix",
  "rovodev",
  "roo",
  "tabnine-cli",
  "terramind",
  "tinycloud",
  "trae",
  "trae-cn",
  "windsurf",
  "zcode",
  "zencoder",
  "zenflow",
  "neovate",
  "pochi",
  "adal",
] as const;

export interface Preferences {
  theme: Theme;
  targetLanguage: TargetLanguage;
  translationProxy: string;
  agents: string[];
  copy: boolean;
}

const KEY = "skill-deck-preferences";

export function defaultLanguage(locale: string): TargetLanguage {
  const normalized = locale.toLowerCase();
  if (normalized.startsWith("zh-tw") || normalized.startsWith("zh-hk"))
    return "zh-Hant";
  const language = normalized.split("-")[0];
  return languages.some(([code]) => code.toLowerCase() === language)
    ? (language as TargetLanguage)
    : normalized.startsWith("zh")
      ? "zh-Hans"
      : "en";
}

export function loadPreferences(locale = navigator.language): Preferences {
  const defaults: Preferences = {
    theme: "system",
    targetLanguage: defaultLanguage(locale),
    translationProxy: "",
    agents: [],
    copy: false,
  };
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(KEY) ?? "null");
    if (!saved || typeof saved !== "object") return defaults;
    const value = saved as Partial<Preferences>;
    return {
      theme: themes.includes(value.theme as Theme)
        ? (value.theme as Theme)
        : defaults.theme,
      targetLanguage: languages.some(([code]) => code === value.targetLanguage)
        ? (value.targetLanguage as TargetLanguage)
        : defaults.targetLanguage,
      translationProxy:
        typeof value.translationProxy === "string" &&
        validateTranslationProxy(value.translationProxy) === null
          ? value.translationProxy
          : "",
      agents: Array.isArray(value.agents)
        ? value.agents.filter(
            (agent): agent is string =>
              typeof agent === "string" &&
              agentOptions.includes(agent as never),
          )
        : [],
      copy: value.copy === true,
    };
  } catch {
    return defaults;
  }
}

export function validateTranslationProxy(value: string): string | null {
  if (!value) return null;
  if (new TextEncoder().encode(value).length > 2_048) return "too-long";
  try {
    const url = new URL(value);
    const authority = value.split("://", 2)[1];
    return (url.protocol === "http:" || url.protocol === "https:") &&
      Boolean(url.host) &&
      Boolean(authority) &&
      !/[/?#]/.test(authority) &&
      !url.username &&
      !url.password &&
      url.pathname === "/" &&
      !url.search &&
      !url.hash
      ? null
      : "invalid";
  } catch {
    return "invalid";
  }
}

export function savePreferences(preferences: Preferences) {
  localStorage.setItem(KEY, JSON.stringify(preferences));
}

export function resolvedTheme(
  theme: Theme,
  dark: boolean,
): Exclude<Theme, "system"> {
  return theme === "system" ? (dark ? "dark" : "light") : theme;
}
