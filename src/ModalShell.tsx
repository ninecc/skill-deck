import { useEffect, useRef, type ReactNode } from "react";

interface Props {
  children: ReactNode;
  labelledBy: string;
  onClose: () => void;
  returnFocus?: HTMLElement | null;
  fallbackFocus?: string;
  initialFocus?: string;
  className?: string;
}

export default function ModalShell({
  children,
  labelledBy,
  onClose,
  returnFocus,
  fallbackFocus,
  initialFocus,
  className = "",
}: Props) {
  const dialog = useRef<HTMLElement>(null);
  const close = useRef(onClose);
  useEffect(() => {
    close.current = onClose;
  }, [onClose]);

  useEffect(() => {
    const node = dialog.current;
    if (!node) return;
    const focusable = () =>
      Array.from(
        node.querySelectorAll<HTMLElement>(
          'button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])',
        ),
      );
    (initialFocus
      ? node.querySelector<HTMLElement>(initialFocus)
      : null
    )?.focus();
    if (!node.contains(document.activeElement)) focusable()[0]?.focus();
    const keydown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        close.current();
        return;
      }
      if (event.key !== "Tab") return;
      const items = focusable();
      if (!items.length) return;
      const first = items[0];
      const last = items[items.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    node.addEventListener("keydown", keydown);
    return () => {
      node.removeEventListener("keydown", keydown);
      if (returnFocus?.isConnected) returnFocus.focus();
      else if (fallbackFocus)
        document.querySelector<HTMLElement>(fallbackFocus)?.focus();
    };
  }, [fallbackFocus, initialFocus, returnFocus]);

  return (
    <div className="sheet-backdrop" role="presentation">
      <section
        ref={dialog}
        className={`settings-sheet ${className}`}
        role="dialog"
        aria-modal="true"
        aria-labelledby={labelledBy}
      >
        {children}
      </section>
    </div>
  );
}
