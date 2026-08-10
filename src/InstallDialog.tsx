import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  commandErrorCode,
  commandErrorMessage,
  commitInstall,
  planInstall,
  type Agent,
  type InstallPlan,
  type Inventory,
  type ManagedSkillPackage,
} from "./api";
import type { Messages } from "./i18n";

interface InstallDialogProps {
  copy: Messages;
  inventory: Inventory;
  skill: ManagedSkillPackage;
  onClose: () => void;
  onCommitted: (message: string) => void;
}

const agents: Agent[] = ["codex", "claude"];

export default function InstallDialog({
  copy,
  inventory,
  skill,
  onClose,
  onCommitted,
}: InstallDialogProps) {
  const [selected, setSelected] = useState<Agent[]>([]);
  const [createMissingRoots, setCreateMissingRoots] = useState(false);
  const [plan, setPlan] = useState<InstallPlan | null>(null);
  const [copyFallback, setCopyFallback] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    dialog?.showModal();
    return () => dialog?.close();
  }, []);

  const missingSelectedRoot = selected.some(
    (agent) =>
      !inventory.targets.find(
        (target) => target.agent === agent && !target.legacy,
      )?.exists,
  );

  function preview(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    void planInstall(skill.id, selected, createMissingRoots)
      .then(setPlan)
      .catch((failure: unknown) =>
        setError(
          commandErrorMessage(failure, copy.errors) ?? copy.unknownError,
        ),
      )
      .finally(() => setBusy(false));
  }

  function commit(confirmCopyFallback: boolean) {
    if (!plan) return;
    setBusy(true);
    setError(null);
    const nextPlan = confirmCopyFallback
      ? planInstall(skill.id, selected, createMissingRoots)
      : Promise.resolve(plan);
    void nextPlan
      .then((value) => commitInstall(value.id, confirmCopyFallback))
      .then(() => {
        onCommitted(copy.restartFallback);
        onClose();
      })
      .catch((failure: unknown) => {
        if (commandErrorCode(failure) === "copy_fallback_required") {
          setCopyFallback(true);
        }
        setError(
          commandErrorMessage(failure, copy.errors) ?? copy.unknownError,
        );
      })
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
          <p className="section-kicker">{skill.name}</p>
          <h2>{copy.installTitle}</h2>
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
          <fieldset className="target-fieldset">
            <legend>{copy.installTargets}</legend>
            {agents.map((agent) => {
              const installed = skill.installations.some(
                (installation) => installation.agent === agent,
              );
              const target = inventory.targets.find(
                (item) => item.agent === agent && !item.legacy,
              );
              return (
                <label key={agent}>
                  <input
                    type="checkbox"
                    disabled={installed}
                    checked={selected.includes(agent)}
                    onChange={(event) =>
                      setSelected((current) =>
                        event.target.checked
                          ? [...current, agent]
                          : current.filter((value) => value !== agent),
                      )
                    }
                  />
                  <span>
                    <strong>
                      {agent === "codex" ? "Codex" : "Claude Code"}
                    </strong>
                    <small>
                      {installed
                        ? copy.alreadyInstalled
                        : (target?.root ?? copy.notCreated)}
                    </small>
                  </span>
                </label>
              );
            })}
          </fieldset>
          {missingSelectedRoot && (
            <label className="confirm-row">
              <input
                type="checkbox"
                checked={createMissingRoots}
                onChange={(event) =>
                  setCreateMissingRoots(event.target.checked)
                }
              />
              {copy.createMissingRoots}
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
              className="primary"
              type="submit"
              disabled={
                busy ||
                selected.length === 0 ||
                (missingSelectedRoot && !createMissingRoots)
              }
            >
              {busy ? copy.validating : copy.previewInstall}
            </button>
          </div>
        </form>
      ) : (
        <div>
          <ul className="install-preview">
            {plan.targets.map((target) => (
              <li key={target.agent}>
                <strong>
                  {target.agent === "codex" ? "Codex" : "Claude Code"}
                </strong>
                <span>{target.logicalPath}</span>
                <small>{target.preferredMode}</small>
              </li>
            ))}
          </ul>
          {error && <p className="inline-error">{error}</p>}
          <div className="dialog-actions">
            <button
              className="secondary"
              type="button"
              disabled={busy}
              onClick={() => {
                setPlan(null);
                setCopyFallback(false);
                setError(null);
              }}
            >
              {copy.back}
            </button>
            <button
              className={copyFallback ? "danger" : "primary"}
              type="button"
              disabled={busy}
              onClick={() => commit(copyFallback)}
            >
              {busy
                ? copy.installing
                : copyFallback
                  ? copy.confirmCopyFallback
                  : copy.installAction}
            </button>
          </div>
        </div>
      )}
    </dialog>
  );
}
