import { useEffect, useRef, useState } from "react";
import {
  commandErrorMessage,
  commitDetach,
  commitRemoveLibrary,
  commitUninstall,
  planDetach,
  planRemoveLibrary,
  planUninstall,
  type Agent,
  type DetachPlan,
  type ManagedSkillPackage,
  type RemoveLibraryPlan,
  type UninstallPlan,
} from "./api";
import type { Messages } from "./i18n";

export type LifecycleAction =
  | { mode: "uninstall" | "detach"; skill: ManagedSkillPackage; agent: Agent }
  | { mode: "remove"; skill: ManagedSkillPackage };

interface LifecycleDialogProps {
  action: LifecycleAction;
  copy: Messages;
  onClose: () => void;
  onCommitted: (message: string) => void;
}

type Plan = UninstallPlan | DetachPlan | RemoveLibraryPlan;

export default function LifecycleDialog({
  action,
  copy,
  onClose,
  onCommitted,
}: LifecycleDialogProps) {
  const [plan, setPlan] = useState<Plan | null>(null);
  const [confirmation, setConfirmation] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(true);
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    dialog?.showModal();
    const request =
      action.mode === "remove"
        ? planRemoveLibrary(action.skill.id)
        : action.mode === "detach"
          ? planDetach(action.skill.id, action.agent)
          : planUninstall(action.skill.id, action.agent);
    void request
      .then(setPlan)
      .catch((failure: unknown) =>
        setError(
          commandErrorMessage(failure, copy.errors) ?? copy.unknownError,
        ),
      )
      .finally(() => setBusy(false));
    return () => dialog?.close();
  }, [action, copy.errors, copy.unknownError]);

  function commit() {
    if (!plan) return;
    setBusy(true);
    setError(null);
    const request =
      action.mode === "remove"
        ? commitRemoveLibrary(plan.id, confirmation)
        : action.mode === "detach"
          ? commitDetach(plan.id)
          : commitUninstall(plan.id);
    void request
      .then(() => {
        onCommitted(copy.restartFallback);
        onClose();
      })
      .catch((failure: unknown) =>
        setError(
          commandErrorMessage(failure, copy.errors) ?? copy.unknownError,
        ),
      )
      .finally(() => setBusy(false));
  }

  const title =
    action.mode === "remove"
      ? copy.removeLibraryTitle
      : action.mode === "detach"
        ? copy.detachTitle
        : copy.uninstallTitle;

  return (
    <dialog
      className="import-dialog"
      ref={dialogRef}
      onCancel={(event) => (busy ? event.preventDefault() : onClose())}
    >
      <div className="dialog-heading">
        <div>
          <p className="section-kicker">{action.skill.name}</p>
          <h2>{title}</h2>
        </div>
        <button
          className="icon-button"
          type="button"
          disabled={busy}
          onClick={onClose}
        >
          <span aria-hidden="true">×</span>
          <span className="sr-only">{copy.close}</span>
        </button>
      </div>
      <div>
        {busy && !plan ? (
          <p className="field-note">{copy.loading}</p>
        ) : plan ? (
          <div className="disclosure-card">
            <p className="section-kicker">{copy.changePreview}</p>
            <p>{"logicalPath" in plan ? plan.logicalPath : plan.libraryPath}</p>
            {action.mode === "detach" && <p>{copy.detachKeepsFiles}</p>}
            {action.mode === "uninstall" && <p>{copy.uninstallOwnedOnly}</p>}
            {action.mode === "remove" && "bytes" in plan && (
              <>
                <p>
                  {copy.removeBytes}: {formatBytes(plan.bytes)}
                </p>
                <p>
                  {copy.exportBeforeRemove}: {plan.exportCurrentPath}
                </p>
                {plan.localSnapshotLastCopyWarning && (
                  <p>{copy.localSnapshotWarning}</p>
                )}
              </>
            )}
          </div>
        ) : null}

        {action.mode === "remove" && plan && (
          <label className="field confirm-name">
            <span>
              {copy.typeExactName.replace("{name}", action.skill.name)}
            </span>
            <input
              value={confirmation}
              onChange={(event) => setConfirmation(event.target.value)}
              spellCheck={false}
            />
          </label>
        )}
        {error && <p className="inline-error">{error}</p>}
        <div className="dialog-actions">
          <button
            className="secondary"
            type="button"
            disabled={busy}
            onClick={onClose}
          >
            {copy.cancel}
          </button>
          <button
            className="danger"
            type="button"
            disabled={
              busy ||
              !plan ||
              (action.mode === "remove" && confirmation !== action.skill.name)
            }
            onClick={commit}
          >
            {busy ? copy.saving : title}
          </button>
        </div>
      </div>
    </dialog>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MiB`;
}
