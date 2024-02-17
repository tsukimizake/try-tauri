import { invoke } from "@tauri-apps/api/tauri";
import { Elm } from "./elm/Main.elm";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;

var app = Elm.Main.init({
  node: document.getElementById("myapp"),
});

async function greet() {
  if (greetMsgEl && greetInputEl) {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    greetMsgEl.textContent = await invoke("greet", {
      name: greetInputEl.value,
    });
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
});

app.ports.readStlFile.subscribe(async function () {
  await invoke("read_stl_file");
});
