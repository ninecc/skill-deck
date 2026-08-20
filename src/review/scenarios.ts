import type {
  FileContent,
  FileEntry,
  InstalledSkill,
  RuntimeStatus,
} from "../api";
import type { Theme, UiLocale } from "../preferences";

export const reviewScenarioIds = [
  "shell-ready",
  "shell-loading",
  "shell-empty",
  "shell-long",
  "shell-zh",
] as const;

export type ReviewScenarioId = (typeof reviewScenarioIds)[number];

const canonicalSkill: InstalledSkill = {
  name: "ask-matt",
  path: "~/.agents/skills/ask-matt",
  scope: "global",
  agents: ["codex"],
  source: "mattpocock/skills",
  sourceUrl: "https://github.com/mattpocock/skills",
  sourceType: "github",
};

const visibleSkills: InstalledSkill[] = [
  canonicalSkill,
  {
    ...canonicalSkill,
    name: "banner-design",
    path: "~/.agents/skills/banner-design",
    source: "claudekit/skills",
  },
  {
    ...canonicalSkill,
    name: "brand",
    path: "~/.agents/skills/brand",
    source: "claudekit/skills",
  },
  {
    ...canonicalSkill,
    name: "code-review",
    path: "~/.agents/skills/code-review",
    source: "mattpocock/skills",
  },
];

const canonicalInventory = [
  ...visibleSkills,
  ...Array.from({ length: 44 }, (_, index): InstalledSkill => {
    const suffix = String(index + 1).padStart(2, "0");
    return {
      ...canonicalSkill,
      name: `fixture-skill-${suffix}`,
      path: `~/.agents/skills/fixture-skill-${suffix}`,
      source: "skill-deck/review-fixtures",
    };
  }),
];

export const canonicalReviewMarker = "SKILL_DECK_CANONICAL_REVIEW_FIXTURE";

export const canonicalTree: FileEntry[] = [
  {
    path: "SKILL.md",
    name: "SKILL.md",
    level: 1,
    directory: false,
    size: 846,
    viewer: "markdown",
    unsupportedReason: null,
  },
  {
    path: "references",
    name: "references",
    level: 1,
    directory: true,
    size: 0,
    viewer: "unsupported",
    unsupportedReason: null,
  },
  {
    path: "references/checklist.md",
    name: "checklist.md",
    level: 2,
    directory: false,
    size: 312,
    viewer: "markdown",
    unsupportedReason: null,
  },
];

export const canonicalPreview: FileContent = {
  path: "SKILL.md",
  viewer: "markdown",
  size: 846,
  text: `# Ask Matt

Use this Skill when you need a second opinion on TypeScript, React, or API design from a pragmatic engineer.

## How to use it

State the decision clearly, include the constraints that matter, and provide the smallest useful example.

\`\`\`sh
npx skills add mattpocock/skills --skill ask-matt
\`\`\`

## Response shape

- Start with the recommendation.
- Make tradeoffs explicit.
- Keep the next action concrete.
`,
  dataUrl: null,
  translatable: true,
};

const longSkill: InstalledSkill = {
  ...canonicalSkill,
  name: "a-deliberately-long-skill-name-for-layout-pressure",
  path: "~/.agents/skills/a-deliberately-long-skill-name-for-layout-pressure",
  source: "an-organization-with-a-very-long-name/a-repository-with-a-long-name",
};

const chineseSkill: InstalledSkill = {
  ...canonicalSkill,
  name: "界面布局与长文本压力测试技能",
  path: "~/.agents/skills/界面布局与长文本压力测试技能",
  source: "技能仓库/中文示例集合",
};

export interface ReviewScenario {
  id: ReviewScenarioId;
  runtime: RuntimeStatus | "pending";
  tree: FileEntry[];
  preview: FileContent;
  autoSelect: boolean;
  theme: Theme;
  locale: UiLocale;
}

function ready(inventory: InstalledSkill[]): RuntimeStatus {
  return {
    ready: true,
    errorCode: null,
    version: "1.5.22",
    nodeVersion: "22.20.0",
    message: null,
    inventory,
  };
}

export const reviewScenarios: Record<ReviewScenarioId, ReviewScenario> = {
  "shell-ready": {
    id: "shell-ready",
    runtime: ready(canonicalInventory),
    tree: canonicalTree,
    preview: canonicalPreview,
    autoSelect: true,
    theme: "dark",
    locale: "en",
  },
  "shell-loading": {
    id: "shell-loading",
    runtime: "pending",
    tree: [],
    preview: canonicalPreview,
    autoSelect: false,
    theme: "dark",
    locale: "en",
  },
  "shell-empty": {
    id: "shell-empty",
    runtime: ready([]),
    tree: [],
    preview: canonicalPreview,
    autoSelect: false,
    theme: "dark",
    locale: "en",
  },
  "shell-long": {
    id: "shell-long",
    runtime: ready([longSkill, ...canonicalInventory.slice(1)]),
    tree: canonicalTree,
    preview: {
      ...canonicalPreview,
      text: `${canonicalPreview.text}\n${"Long content. ".repeat(80)}`,
    },
    autoSelect: true,
    theme: "light",
    locale: "en",
  },
  "shell-zh": {
    id: "shell-zh",
    runtime: ready([chineseSkill, ...canonicalInventory.slice(1)]),
    tree: canonicalTree,
    preview: {
      ...canonicalPreview,
      text: "# 请教专家\n\n用于检验简体中文界面、长文本换行以及阅读区域的弹性。\n\n## 使用方式\n\n清楚描述决定、约束条件与最小示例。",
    },
    autoSelect: true,
    theme: "dark",
    locale: "zh-CN",
  },
};

export function isReviewScenarioId(
  value: string | null,
): value is ReviewScenarioId {
  return reviewScenarioIds.some((id) => id === value);
}
