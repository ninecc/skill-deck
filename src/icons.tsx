type IconName =
  | "search"
  | "settings"
  | "refresh"
  | "folder"
  | "file"
  | "translate"
  | "trash"
  | "download"
  | "close"
  | "chevron";

const paths: Record<IconName, React.ReactNode> = {
  search: (
    <>
      <circle cx="11" cy="11" r="7" />
      <path d="m20 20-4-4" />
    </>
  ),
  settings: (
    <>
      <circle cx="12" cy="12" r="3" />
      <path d="M12 2v3m0 14v3M2 12h3m14 0h3M5 5l2 2m10 10 2 2M19 5l-2 2M7 17l-2 2" />
    </>
  ),
  refresh: (
    <>
      <path d="M20 7v5h-5" />
      <path d="M19 12a7 7 0 1 0-2 5" />
    </>
  ),
  folder: <path d="M3 6h7l2 2h9v11H3z" />,
  file: <path d="M6 2h8l4 4v16H6zM14 2v5h5" />,
  translate: (
    <>
      <path d="M4 5h9M8.5 3v2m-3 3c2 3 4 5 7 6m0-6c-2 3-4 5-7 6" />
      <path d="m14 21 3-8 3 8m-5-3h4" />
    </>
  ),
  trash: (
    <>
      <path d="M4 7h16M9 7V4h6v3m3 0-1 14H7L6 7" />
    </>
  ),
  download: (
    <>
      <path d="M12 3v12m-4-4 4 4 4-4" />
      <path d="M4 20h16" />
    </>
  ),
  close: <path d="m5 5 14 14M19 5 5 19" />,
  chevron: <path d="m7 10 5 5 5-5" />,
};

// Phosphor Regular-compatible local SVGs: no runtime icon lookup or network path.
export function Icon({ name }: { name: IconName }) {
  return (
    <svg
      className="icon"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {paths[name]}
    </svg>
  );
}
