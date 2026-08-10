import { useEffect, useRef, useState } from "react";
import {
  checkGitUpdate,
  commandErrorMessage,
  commitGitUpdate,
  type GitUpdateCheck,
  type ManagedSkillPackage,
} from "./api";
import type { Messages } from "./i18n";

interface GitUpdateDialogProps {
  copy: Messages;
  skill: ManagedSkillPackage;
  onClose: () => void;
  onCommitted: (message: string) => void;
}

export default function GitUpdateDialog({
  copy,
  skill,
  onClose,
  onCommitted,
}: GitUpdateDialogProps) {
  const [check, setCheck] = useState<GitUpdateCheck | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(true);
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    dialog?.showModal();
    void checkGitUpdate(skill.id)
      .then(setCheck)
      .catch((failure: unknown) =>
        setError(
          commandErrorMessage(failure, copy.errors) ?? copy.unknownError,
        ),
      )
      .finally(() => setBusy(false));
    return () => dialog?.close();
  }, [copy.errors, copy.unknownError, skill.id]);

  function commit() {
    if (!check?.plan) return;
    setBusy(true);
    setError(null);
    void commitGitUpdate(check.plan.id)
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

  return (
    <dialog
      className="import-dialog"
      ref={dialogRef}
      onCancel={(event) => (busy ? event.preventDefault() : onClose())}
    >
      <div className="dialog-heading">
        <div>
          <p className="section-kicker">{copy.publicGit}</p>
          <h2>{copy.gitUpdateTitle}</h2>
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
        {busy && !check ? (
          <p className="field-note">{copy.checkingUpdate}</p>
        ) : check ? (
          <div className="disclosure-card">
            <p className="section-kicker">{copy.updateStatus}</p>
            <h3>{copy[`gitStatus_${check.status}`]}</h3>
            {check.remoteCommitOid && <p>{check.remoteCommitOid}</p>}
            {check.plan && (
              <dl className="facts revision-facts">
                <div>
                  <dt>{copy.scripts}</dt>
                  <dd>{changeCount(check.plan.changes.scripts)}</dd>
                </div>
                <div>
                  <dt>{copy.references}</dt>
                  <dd>{changeCount(check.plan.changes.references)}</dd>
                </div>
                <div>
                  <dt>{copy.unknownFields}</dt>
                  <dd>{changeCount(check.plan.changes.unknownFields)}</dd>
                </div>
              </dl>
            )}
          </div>
        ) : null}
        {error && <p className="inline-error">{error}</p>}
        <div className="dialog-actions">
          <button
            className="secondary"
            type="button"
            disabled={busy}
            onClick={onClose}
          >
            {copy.close}
          </button>
          {check?.status === "fast_forward" && check.plan && (
            <button
              className="primary"
              type="button"
              disabled={busy}
              onClick={commit}
            >
              {busy ? copy.saving : copy.updateAction}
            </button>
          )}
        </div>
      </div>
    </dialog>
  );
}

function changeCount(change: { added: string[]; removed: string[] }): string {
  return `+${change.added.length} / −${change.removed.length}`;
}
