import { useEffect, useRef, useState, type FormEvent } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  commandErrorMessage,
  commitExportRevision,
  commitReplaceLocalRevision,
  commitRestoreInstallation,
  commitRollbackRevision,
  planExportRevision,
  planReplaceLocalRevision,
  planRestoreInstallation,
  planRollbackRevision,
  type Agent,
  type ExportRevisionPlan,
  type ManagedSkillPackage,
  type ReplaceRevisionPlan,
  type RestoreInstallationPlan,
  type RollbackRevisionPlan,
} from "./api";
import type { Messages } from "./i18n";

export type RevisionAction =
  | { mode: "replace" | "export" | "rollback"; skill: ManagedSkillPackage }
  | { mode: "restore"; skill: ManagedSkillPackage; agent: Agent };

interface RevisionDialogProps {
  action: RevisionAction;
  copy: Messages;
  onClose: () => void;
  onCommitted: (message: string) => void;
}

type Plan =
  | ReplaceRevisionPlan
  | ExportRevisionPlan
  | RollbackRevisionPlan
  | RestoreInstallationPlan;

export default function RevisionDialog({
  action,
  copy,
  onClose,
  onCommitted,
}: RevisionDialogProps) {
  const automatic = action.mode === "rollback" || action.mode === "restore";
  const [path, setPath] = useState("");
  const [plan, setPlan] = useState<Plan | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(automatic);
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    dialog?.showModal();
    if (action.mode === "rollback") {
      void planRollbackRevision(action.skill.id)
        .then(setPlan)
        .catch((failure: unknown) =>
          setError(
            commandErrorMessage(failure, copy.errors) ?? copy.unknownError,
          ),
        )
        .finally(() => setBusy(false));
    } else if (action.mode === "restore") {
      void planRestoreInstallation(action.skill.id, action.agent)
        .then(setPlan)
        .catch((failure: unknown) =>
          setError(
            commandErrorMessage(failure, copy.errors) ?? copy.unknownError,
          ),
        )
        .finally(() => setBusy(false));
    }
    return () => dialog?.close();
  }, [action, copy.errors, copy.unknownError]);

  function preview(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    const request =
      action.mode === "replace"
        ? planReplaceLocalRevision(action.skill.id, path.trim())
        : planExportRevision(action.skill.id, path.trim());
    void request
      .then(setPlan)
      .catch((failure: unknown) =>
        setError(
          commandErrorMessage(failure, copy.errors) ?? copy.unknownError,
        ),
      )
      .finally(() => setBusy(false));
  }

  function commit() {
    if (!plan) return;
    setBusy(true);
    setError(null);
    if (action.mode === "export") {
      void commitExportRevision(plan.id)
        .then((result) => {
          onCommitted(`${copy.exportComplete}: ${result.destination}`);
          onClose();
        })
        .catch(showError)
        .finally(() => setBusy(false));
      return;
    }
    const request =
      action.mode === "replace"
        ? commitReplaceLocalRevision(plan.id)
        : action.mode === "rollback"
          ? commitRollbackRevision(plan.id)
          : commitRestoreInstallation(plan.id, true);
    void request
      .then(() => {
        onCommitted(copy.restartFallback);
        onClose();
      })
      .catch(showError)
      .finally(() => setBusy(false));
  }

  function showError(failure: unknown) {
    setError(commandErrorMessage(failure, copy.errors) ?? copy.unknownError);
  }

  const title = copy[`${action.mode}RevisionTitle`];

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
        {!automatic && !plan && (
          <form className="embedded-form" onSubmit={preview}>
            <label className="field">
              <span>
                {action.mode === "replace"
                  ? copy.replacementPath
                  : copy.exportDestination}
              </span>
              <input
                required
                value={path}
                onChange={(event) => setPath(event.target.value)}
                spellCheck={false}
              />
            </label>
            {action.mode === "replace" && (
              <button
                className="secondary choose-folder"
                type="button"
                onClick={() =>
                  void open({ directory: true, multiple: false }).then(
                    (selected) => {
                      if (typeof selected === "string") setPath(selected);
                    },
                    showError,
                  )
                }
              >
                {copy.chooseFolder}
              </button>
            )}
            <div className="dialog-actions">
              <button
                className="secondary"
                type="button"
                disabled={busy}
                onClick={onClose}
              >
                {copy.cancel}
              </button>
              <button className="primary" type="submit" disabled={busy}>
                {busy ? copy.validating : copy.preview}
              </button>
            </div>
          </form>
        )}
        {busy && !plan && automatic && (
          <p className="field-note">{copy.loading}</p>
        )}
        {plan && (
          <div className="disclosure-card">
            <p className="section-kicker">{copy.changePreview}</p>
            {"changes" in plan && (
              <dl className="facts revision-facts">
                <div>
                  <dt>{copy.scripts}</dt>
                  <dd>{changeCount(plan.changes.scripts)}</dd>
                </div>
                <div>
                  <dt>{copy.references}</dt>
                  <dd>{changeCount(plan.changes.references)}</dd>
                </div>
                <div>
                  <dt>{copy.unknownFields}</dt>
                  <dd>{changeCount(plan.changes.unknownFields)}</dd>
                </div>
              </dl>
            )}
            {"willOverwrite" in plan && <p>{copy.restoreOverwriteWarning}</p>}
            {"destination" in plan && <p>{plan.destination}</p>}
          </div>
        )}
        {error && <p className="inline-error">{error}</p>}
        {plan && (
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
              className={action.mode === "restore" ? "danger" : "primary"}
              type="button"
              disabled={busy}
              onClick={commit}
            >
              {busy ? copy.saving : title}
            </button>
          </div>
        )}
      </div>
    </dialog>
  );
}

function changeCount(change: { added: string[]; removed: string[] }): string {
  return `+${change.added.length} / −${change.removed.length}`;
}
