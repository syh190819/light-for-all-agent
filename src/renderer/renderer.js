const lamps = [...document.querySelectorAll(".lamp")];
const { core, event: tauriEvent, window: tauriWindow } = window.__TAURI__;
const appWindow = tauriWindow.getCurrentWindow();

function setOrientation(orientation = "horizontal") {
  document.documentElement.dataset.orientation = orientation;
}

function paint({ currentState }) {
  const mode = currentState.mode || "idle";
  lamps.forEach((lamp) =>
    lamp.classList.toggle("active", lamp.dataset.mode === mode)
  );
}

window.addEventListener("contextmenu", (event) => {
  event.preventDefault();
  core.invoke("show_context_menu", { x: event.clientX, y: event.clientY });
});

window.addEventListener("pointerdown", async (event) => {
  if (event.button !== 0) return;
  try {
    await appWindow.startDragging();
  } catch (error) {
    console.error("拖动窗口失败", error);
  }
});

core.invoke("get_state").then((state) => {
  setOrientation(state.orientation);
  paint(state);
});

tauriEvent.listen("status-changed", ({ payload }) => paint(payload));
tauriEvent.listen("orientation-changed", ({ payload }) => setOrientation(payload));
