// Settings window controller.
//
// Loads current settings + integration status on open, lets the user edit
// the small set of free-tier knobs, and saves via the `update_settings`
// command. The shell-integration toggle is wired to dedicated commands so
// registry changes happen immediately rather than only on save.

const { invoke } = window.__TAURI__.core;

const $ = (id) => document.getElementById(id);

const ctxToggle = $("ctx-toggle");
const ctxStatus = $("ctx-status");
const outputFolder = $("output-folder-display");
const pickFolderBtn = $("pick-folder");
const outputMode = $("output-mode");
const preserveMetadata = $("preserve-metadata");
const saveBtn = $("save-btn");
const resetBtn = $("reset-btn");
const githubBtn = $("github-btn");

let currentSettings = null;

async function refreshIntegration() {
  // The toggle and status text are independent of `loadSettings`. Any
  // failure here must surface in the UI rather than leaving the placeholder
  // "Checking integration status…" sitting forever.
  try {
    const status = await invoke("shell_integration_status");
    if (!status.supported) {
      ctxToggle.disabled = true;
      ctxToggle.textContent = "Unsupported";
      ctxStatus.textContent = "Context-menu integration is only available on Windows.";
      return;
    }
    ctxToggle.disabled = false;
    ctxToggle.textContent = status.installed ? "Disable" : "Enable";
    ctxStatus.textContent = status.installed
      ? "Context-menu entries are registered for the current user."
      : "Context-menu entries are not registered.";
  } catch (err) {
    ctxToggle.disabled = false;
    ctxToggle.textContent = "Enable";
    ctxStatus.textContent = `Could not read integration status: ${err}`;
    console.error("shell_integration_status failed", err);
  }
}

async function loadSettings() {
  try {
    currentSettings = (await invoke("get_settings")) || {};
  } catch (err) {
    currentSettings = {};
    console.error("get_settings failed", err);
  }
  outputFolder.textContent = currentSettings.output_folder || "Same folder as input";
  outputMode.value = currentSettings.output_mode || "rename";
  preserveMetadata.checked = !!currentSettings.preserve_metadata;
}

ctxToggle.addEventListener("click", async () => {
  const original = ctxToggle.textContent;
  ctxToggle.disabled = true;
  ctxToggle.textContent = "Working…";
  try {
    const status = await invoke("shell_integration_status");
    if (status.installed) {
      await invoke("uninstall_shell_integration");
    } else {
      await invoke("install_shell_integration");
    }
  } catch (err) {
    ctxStatus.textContent = String(err);
    ctxToggle.disabled = false;
    ctxToggle.textContent = original;
    return;
  }
  await refreshIntegration();
});

pickFolderBtn.addEventListener("click", async () => {
  const folder = await invoke("pick_output_folder");
  if (typeof folder === "string" && folder.length > 0) {
    currentSettings.output_folder = folder;
    outputFolder.textContent = folder;
  }
});

saveBtn.addEventListener("click", async () => {
  if (!currentSettings) return;
  const next = {
    ...currentSettings,
    output_mode: outputMode.value,
    preserve_metadata: preserveMetadata.checked,
  };
  try {
    currentSettings = await invoke("update_settings", { newSettings: next });
    saveBtn.textContent = "Saved";
    setTimeout(() => (saveBtn.textContent = "Save"), 1500);
  } catch (err) {
    saveBtn.textContent = "Failed";
    saveBtn.classList.add("danger");
    console.error(err);
  }
});

resetBtn.addEventListener("click", async () => {
  try {
    currentSettings = await invoke("reset_settings");
  } catch (err) {
    console.error("reset_settings failed", err);
  }
  await loadSettings();
  await refreshIntegration();
});

if (githubBtn) {
  githubBtn.addEventListener("click", async () => {
    try {
      await invoke("open_external_url", { url: "https://github.com/neikiri/xolariq" });
    } catch (err) {
      console.error("open_external_url failed", err);
    }
  });
}

// Run independently so that a failure in one path (e.g. settings.json
// missing on first launch) does not leave the integration status stuck
// on "Checking integration status…" forever.
loadSettings().catch((err) => console.error("loadSettings failed", err));
refreshIntegration().catch((err) => console.error("refreshIntegration failed", err));
