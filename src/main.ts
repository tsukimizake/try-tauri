import { invoke } from "@tauri-apps/api/core";
import { Elm } from "./elm/Main.elm";
import { listen } from "@tauri-apps/api/event";

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

await listen("tauri_msg", (e) => {
  app.ports.fromTauriMsg.send(e.payload);
});

type ToTauriMsg = {
  [key: string]: string;
}

app.ports.toTauriMsg.subscribe(async function(json: ToTauriMsg) {
  await invoke("from_elm", { args: JSON.stringify(json) });
});
