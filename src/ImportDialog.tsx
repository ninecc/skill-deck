import { useEffect, useRef, useState, type FormEvent } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  commandErrorMessage,
  commitAddLocalSkill,
  commitGitImport,
  planAddLocalSkill,
  planGitImport,
  type AddToLibraryPlan,
  type GitImportPlan,
} from "./api";
import type { Messages } from "./i18n";

interface ImportDialogProps {
  copy: Messages;
  onClose: () => void;
  onCommitted: () => void;
}

export default function ImportDialog({
  copy,
  onClose,
  onCommitted,
}: ImportDialogProps) {
  const [source, setSource] = useState<"local" | "git">("local");
  const [path, setPath] = useState("");
  const [repositoryUrl, setRepositoryUrl] = useState("");
  const [subpath, setSubpath] = useState(".");
  const [trackedBranch, setTrackedBranch] = useState("main");
  const [plan, setPlan] = useState<AddToLibraryPlan | GitImportPlan | null>(
    null,
  );
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    dialog?.showModal();
    return () => dialog?.close();
  }, []);

  function preview(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    const request =
      source === "local"
        ? planAddLocalSkill(path.trim())
        : planGitImport(
            repositoryUrl.trim(),
            subpath.trim(),
            trackedBranch.trim(),
          );
    void request
      .then(setPlan)
      .catch((failure: unknown) => {
        setError(
          commandErrorMessage(failure, copy.errors) ?? copy.unknownError,
        );
      })
      .finally(() => setBusy(false));
  }

  function commit() {
    if (!plan) return;
    setBusy(true);
    setError(null);
    const request =
      source === "local"
        ? commitAddLocalSkill(plan.id)
        : commitGitImport(plan.id);
    void request
      .then(() => {
        onCommitted();
        onClose();
      })
      .catch((failure: unknown) => {
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
          <p className="section-kicker">
            {source === "local" ? copy.localSnapshot : copy.publicGit}
          </p>
          <h2>{copy.importTitle}</h2>
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
          <fieldset className="source-switch">
            <legend>{copy.sourceType}</legend>
            <label>
              <input
                type="radio"
                name="source"
                checked={source === "local"}
                onChange={() => setSource("local")}
              />
              {copy.localSnapshot}
            </label>
            <label>
              <input
                type="radio"
                name="source"
                checked={source === "git"}
                onChange={() => setSource("git")}
              />
              {copy.publicGit}
            </label>
          </fieldset>
          {source === "local" ? (
            <>
              <label className="field">
                <span>{copy.localPath}</span>
                <input
                  required
                  value={path}
                  onChange={(event) => setPath(event.target.value)}
                  placeholder={copy.localPathPlaceholder}
                  spellCheck={false}
                />
              </label>
              <button
                className="secondary choose-folder"
                type="button"
                onClick={() =>
                  void open({ directory: true, multiple: false }).then(
                    (selected) => {
                      if (typeof selected === "string") setPath(selected);
                    },
                    (failure: unknown) =>
                      setError(
                        commandErrorMessage(failure, copy.errors) ??
                          copy.unknownError,
                      ),
                  )
                }
              >
                {copy.chooseFolder}
              </button>
            </>
          ) : (
            <div className="git-fields">
              <label className="field">
                <span>{copy.repositoryUrl}</span>
                <input
                  required
                  type="url"
                  value={repositoryUrl}
                  onChange={(event) => setRepositoryUrl(event.target.value)}
                  placeholder="https://github.com/org/repository.git"
                  spellCheck={false}
                />
              </label>
              <label className="field">
                <span>{copy.skillSubpath}</span>
                <input
                  required
                  value={subpath}
                  onChange={(event) => setSubpath(event.target.value)}
                  spellCheck={false}
                />
              </label>
              <label className="field">
                <span>{copy.trackedBranch}</span>
                <input
                  required
                  value={trackedBranch}
                  onChange={(event) => setTrackedBranch(event.target.value)}
                  spellCheck={false}
                />
              </label>
            </div>
          )}
          <p className="field-note">{copy.zeroTargets}</p>
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
            <button className="primary" type="submit" disabled={busy}>
              {busy ? copy.validating : copy.preview}
            </button>
          </div>
        </form>
      ) : (
        <div>
          <div className="disclosure-card">
            <p className="section-kicker">{copy.structuralValidation}</p>
            <h3>{plan.skill.metadata.name}</h3>
            <p>{plan.skill.metadata.description}</p>
            <dl className="facts">
              <div>
                <dt>{copy.files}</dt>
                <dd>{plan.skill.resources.fileCount}</dd>
              </div>
              <div>
                <dt>{copy.size}</dt>
                <dd>{formatBytes(plan.skill.resources.packageBytes)}</dd>
              </div>
              <div>
                <dt>{copy.scripts}</dt>
                <dd>{plan.skill.scripts.length}</dd>
              </div>
              <div>
                <dt>{copy.references}</dt>
                <dd>{plan.skill.references.length}</dd>
              </div>
            </dl>
          </div>
          <p className="field-note">{copy.disclosureNote}</p>
          {error && <p className="inline-error">{error}</p>}
          <div className="dialog-actions">
            <button
              className="secondary"
              type="button"
              disabled={busy}
              onClick={() => setPlan(null)}
            >
              {copy.back}
            </button>
            <button
              className="primary"
              type="button"
              disabled={busy}
              onClick={commit}
            >
              {busy ? copy.saving : copy.addToLibrary}
            </button>
          </div>
        </div>
      )}
    </dialog>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MiB`;
}
