import { save } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef, useState } from "react";
import {
  commandErrorMessage,
  commitDiagnosticsExport,
  inventoryDiagnosticMessage,
  planDiagnosticsExport,
  type DiagnosticsExportPlan,
  type Inventory,
  type StateStatus,
} from "./api";
import type { Messages } from "./i18n";

interface SettingsDialogProps {
  copy: Messages;
  inventory: Inventory;
  stateStatus: StateStatus;
  onClose: () => void;
}

export default function SettingsDialog({
  copy,
  inventory,
  stateStatus,
  onClose,
}: SettingsDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [diagnosticsPlan, setDiagnosticsPlan] =
    useState<DiagnosticsExportPlan | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    dialog?.showModal();
    return () => dialog?.close();
  }, []);

  function previewDiagnostics() {
    setError(null);
    void save({
      defaultPath: "skill-deck-diagnostics.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    }).then((destination) => {
      if (!destination) return;
      setBusy(true);
      void planDiagnosticsExport(destination)
        .then(setDiagnosticsPlan)
        .catch(showError)
        .finally(() => setBusy(false));
    }, showError);
  }

  function exportDiagnostics() {
    if (!diagnosticsPlan) return;
    setBusy(true);
    setError(null);
    void commitDiagnosticsExport(diagnosticsPlan.id)
      .then((result) => {
        setMessage(`${copy.exportComplete}: ${result.destination}`);
        setDiagnosticsPlan(null);
      })
      .catch(showError)
      .finally(() => setBusy(false));
  }

  function showError(failure: unknown) {
    setError(commandErrorMessage(failure, copy.errors) ?? copy.unknownError);
  }

  return (
    <dialog
      className="import-dialog settings-dialog"
      ref={dialogRef}
      onCancel={(event) => (busy ? event.preventDefault() : onClose())}
    >
      <div className="dialog-heading">
        <div>
          <p className="section-kicker">{copy.settingsDiagnostics}</p>
          <h2>{copy.pathsTitle}</h2>
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
        <dl className="path-list">
          {inventory.targets.map((target) => (
            <div key={`${target.agent}:${target.root}`}>
              <dt>
                {target.agent}
                {target.legacy ? ` · ${copy.legacy}` : ""}
              </dt>
              <dd>{target.root}</dd>
              <dd className={target.exists ? "path-ok" : "path-missing"}>
                {target.exists ? copy.detected : copy.notCreated}
              </dd>
            </div>
          ))}
        </dl>
        <div className="state-diagnostic">
          <span>{copy.stateMode}</span>
          <strong>{stateStatus.mode.replaceAll("_", " ")}</strong>
          {stateStatus.diagnostic && <p>{stateStatus.diagnostic}</p>}
        </div>
        {inventory.attentionEntries.length > 0 && (
          <div className="state-diagnostic">
            <strong>
              {copy.needsAttention}: {inventory.attentionEntries.length}
            </strong>
            <ul className="attention-list">
              {inventory.attentionEntries.map((entry) => (
                <li key={`${entry.agent}:${entry.logicalPath}`}>
                  <strong>
                    {entry.agent} · {copy[`attention_${entry.kind}`]}
                  </strong>
                  <span>{entry.logicalPath}</span>
                  <span>
                    {inventoryDiagnosticMessage(
                      entry.diagnostic,
                      entry.logicalPath,
                      copy.errors,
                    )}
                  </span>
                </li>
              ))}
            </ul>
          </div>
        )}
        {diagnosticsPlan && (
          <div className="disclosure-card diagnostics-preview">
            <p className="section-kicker">{copy.diagnosticsPreview}</p>
            <p>
              {copy.managed}: {diagnosticsPlan.report.managedPackageCount} ·{" "}
              {copy.external}:{" "}
              {diagnosticsPlan.report.externalInstallationCount} ·{" "}
              {copy.needsAttention}: {diagnosticsPlan.report.attentionCount}
            </p>
            <p>
              {copy.omitted}: {diagnosticsPlan.report.omitted.join(", ")}
            </p>
            <p>{diagnosticsPlan.report.destination}</p>
          </div>
        )}
        {message && <p className="success-banner dialog-banner">{message}</p>}
        {error && <p className="inline-error">{error}</p>}
        <div className="dialog-actions">
          <button
            className="secondary"
            type="button"
            disabled={busy}
            onClick={previewDiagnostics}
          >
            {copy.previewDiagnostics}
          </button>
          {diagnosticsPlan && (
            <button
              className="primary"
              type="button"
              disabled={busy}
              onClick={exportDiagnostics}
            >
              {busy ? copy.saving : copy.exportDiagnostics}
            </button>
          )}
          <button
            className="primary"
            type="button"
            disabled={busy}
            onClick={onClose}
          >
            {copy.close}
          </button>
        </div>
      </div>
    </dialog>
  );
}
