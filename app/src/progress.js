// Progress window controller.
//
// All wiring is event-driven: the Rust side emits `xolariq:progress` whenever
// the queue moves and the UI mutates DOM in place. We deliberately avoid a
// JS framework — the surface is small enough that it's cheaper to read.

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const currentWindow = window.__TAURI__.window.getCurrentWindow
  ? window.__TAURI__.window.getCurrentWindow()
  : window.__TAURI__.webviewWindow.getCurrentWebviewWindow();

const $ = (id) => document.getElementById(id);

const heading = $("heading");
const counter = $("counter");
const currentFile = $("current-file");
const outputPath = $("output-path");
const bar = $("bar");
const errorCard = $("error-card");
const errorMsg = $("error-message");
const cancelBtn = $("cancel-btn");
const closeBtn = $("close-btn");

let totalJobs = 0;
let finishedJobs = 0;
let failures = 0;

function updateCounter() {
  if (totalJobs > 0) {
    counter.textContent = `${finishedJobs} of ${totalJobs}`;
  } else {
    counter.textContent = "";
  }
}

function setBarPercent(percent) {
  if (percent == null || Number.isNaN(percent)) {
    bar.removeAttribute("value");
  } else {
    bar.value = Math.max(0, Math.min(100, percent * 100));
  }
}

listen("xolariq:progress", ({ payload }) => {
  if (!payload || !payload.kind) return;
  switch (payload.kind) {
    case "queue_started":
      totalJobs = payload.total;
      finishedJobs = 0;
      failures = 0;
      heading.textContent = "Converting…";
      errorCard.hidden = true;
      cancelBtn.hidden = false;
      closeBtn.hidden = true;
      setBarPercent(0);
      updateCounter();
      break;

    case "job_started":
      currentFile.textContent = payload.input;
      outputPath.textContent = `→ ${payload.output}`;
      setBarPercent(0);
      break;

    case "job_progress":
      setBarPercent(payload.percent);
      break;

    case "job_finished":
      finishedJobs += 1;
      updateCounter();
      setBarPercent(1);
      break;

    case "job_failed":
      finishedJobs += 1;
      failures += 1;
      errorCard.hidden = false;
      errorMsg.textContent = payload.error || "Unknown error.";
      updateCounter();
      break;

    case "job_cancelled":
      finishedJobs += 1;
      updateCounter();
      break;

    case "queue_finished":
      heading.textContent =
        payload.failures === 0 && !payload.cancelled
          ? "Done"
          : payload.cancelled
          ? "Cancelled"
          : "Finished with errors";
      cancelBtn.hidden = true;
      closeBtn.hidden = false;
      setBarPercent(payload.failures === 0 && !payload.cancelled ? 1 : null);
      break;

    default:
      break;
  }
});

cancelBtn.addEventListener("click", async () => {
  cancelBtn.disabled = true;
  await invoke("cancel_current_job");
});

closeBtn.addEventListener("click", async () => {
  if (currentWindow && currentWindow.hide) {
    await currentWindow.hide();
  } else if (currentWindow && currentWindow.close) {
    await currentWindow.close();
  } else {
    window.close();
  }
});
