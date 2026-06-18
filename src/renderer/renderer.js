const lamps = [...document.querySelectorAll(".lamp")];
const { core, event: tauriEvent, window: tauriWindow } = window.__TAURI__;
const appWindow = tauriWindow.getCurrentWindow();

function setOrientation(orientation = "horizontal") {
  document.documentElement.dataset.orientation = orientation;
}

function paint({ currentState }) {
  const mode = currentState.mode || "idle";
  // waiting 复用 working(黄)灯位，但通过 data-active-mode 显示为橙色闪烁
  const slotMode = mode === "waiting" ? "working" : mode;
  lamps.forEach((lamp) => {
    const isActive = lamp.dataset.mode === slotMode;
    lamp.classList.toggle("active", isActive);
    if (isActive) {
      lamp.dataset.activeMode = mode;
    } else {
      delete lamp.dataset.activeMode;
    }
  });
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
