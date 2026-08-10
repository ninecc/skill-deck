import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  commandErrorCode,
  commandErrorMessage,
  commitAdoption,
  commitLegacyMigration,
  planAdoption,
  planLegacyMigration,
  type AdoptionPlan,
  type ExternalInstallation,
  type ExternalInstallationIdentity,
  type LegacyMigrationPlan,
} from "./api";
import type { Messages } from "./i18n";

interface AdoptionDialogProps {
  candidates: ExternalInstallation[];
  copy: Messages;
  entry: ExternalInstallation;
  onClose: () => void;
  onCommitted: (message: string) => void;
}

type Plan = AdoptionPlan | LegacyMigrationPlan;

export default function AdoptionDialog({
  candidates,
  copy,
  entry,
  onClose,
  onCommitted,
}: AdoptionDialogProps) {
  const legacy = entry.kind.startsWith("legacy_");
  const [selected, setSelected] = useState(() => [key(entry)]);
  const [plan, setPlan] = useState<Plan | null>(null);
  const [copyFallback, setCopyFallback] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    dialog?.showModal();
    return () => dialog?.close();
  }, []);

  function identities(): ExternalInstallationIdentity[] {
    return candidates
      .filter((candidate) => selected.includes(key(candidate)))
      .map(({ agent, logicalPath }) => ({ agent, logicalPath }));
  }

  function preview(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    const request = legacy
      ? planLegacyMigration(entry.logicalPath)
      : planAdoption(identities());
    void request
      .then(setPlan)
      .catch(showError)
      .finally(() => setBusy(false));
  }

  function commit(confirmCopyFallback: boolean) {
    if (!plan) return;
    setBusy(true);
    setError(null);
    if (legacy) {
      void commitLegacyMigration(plan.id)
        .then(done)
        .catch(showError)
        .finally(() => setBusy(false));
      return;
    }
    const nextPlan = confirmCopyFallback
      ? planAdoption(identities())
      : Promise.resolve(plan as AdoptionPlan);
    void nextPlan
      .then((value) => commitAdoption(value.id, confirmCopyFallback))
      .then(done)
      .catch((failure: unknown) => {
        if (commandErrorCode(failure) === "copy_fallback_required") {
          setCopyFallback(true);
        }
        showError(failure);
      })
      .finally(() => setBusy(false));
  }

  function done() {
    onCommitted(copy.restartFallback);
    onClose();
  }

  function showError(failure: unknown) {
    setError(commandErrorMessage(failure, copy.errors) ?? copy.unknownError);
  }

  return (
    <dialog
      className="import-dialog"
      ref={dialogRef}
      onCancel={(event) => (busy ? event.preventDefault() : onClose())}
    >
      <div className="dialog-heading">
        <div>
          <p className="section-kicker">{copy.externalSource}</p>
          <h2>{legacy ? copy.legacyMigrationTitle : copy.adoptionTitle}</h2>
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
      {!plan ? (
        <form onSubmit={preview}>
          {legacy ? (
            <p className="field-note">{copy.legacyMigrationNote}</p>
          ) : (
            <fieldset className="target-fieldset">
              <legend>{copy.adoptionTargets}</legend>
              {candidates.map((candidate) => (
                <label key={key(candidate)}>
                  <input
                    type="checkbox"
                    checked={selected.includes(key(candidate))}
                    onChange={(event) =>
                      setSelected((current) =>
                        event.target.checked
                          ? [...current, key(candidate)]
                          : current.filter((value) => value !== key(candidate)),
                      )
                    }
                  />
                  <span>
                    <strong>{candidate.agent}</strong>
                    <small>{candidate.logicalPath}</small>
                  </span>
                </label>
              ))}
            </fieldset>
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
              className="primary"
              type="submit"
              disabled={busy || (!legacy && selected.length === 0)}
            >
              {busy ? copy.validating : copy.preview}
            </button>
          </div>
        </form>
      ) : (
        <div>
          <div className="disclosure-card">
            <p className="section-kicker">{copy.changePreview}</p>
            <h3>{plan.name}</h3>
            <p>{copy.adoptionPreservesTarget}</p>
            <p>{"legacyPath" in plan ? plan.legacyPath : plan.libraryPath}</p>
          </div>
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
              className={copyFallback ? "danger" : "primary"}
              type="button"
              disabled={busy}
              onClick={() => commit(copyFallback)}
            >
              {busy
                ? copy.saving
                : copyFallback
                  ? copy.confirmCopyFallback
                  : legacy
                    ? copy.migrateAction
                    : copy.adoptAction}
            </button>
          </div>
        </div>
      )}
    </dialog>
  );
}

function key(entry: ExternalInstallation): string {
  return `${entry.agent}:${entry.logicalPath}`;
}
