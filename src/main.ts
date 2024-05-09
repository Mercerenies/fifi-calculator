
const { invoke } = (window as any).__TAURI__.tauri;

window.addEventListener("DOMContentLoaded", () => {
  document.getElementById("input-box").innerHTML = "<b>A</b>";
});
