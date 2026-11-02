import html from "../html/editor-controls.html?raw";
import { GisstElement } from "./globals";

class EditorControls extends GisstElement {
  private muteButton: HTMLButtonElement | null = null;
  private mutedValue = false;
  private zoomValue = "1";

  connectedCallback() {
    if (this.querySelector(".editor-controls-row")) {
      return;
    }

    const template = document.createElement("template");
    template.innerHTML = html;
    this.append(template.content.cloneNode(true));

    this.muteButton = this.querySelector("[data-action='mute']");
    const zoomButton = this.querySelector("[data-action='zoom']");
    const saveButton = this.querySelector("[data-action='save']");
    const recordButton = this.querySelector("[data-action='record']");
    const stopButton = this.querySelector("[data-action='stop']");

    this.muted = this.mutedValue;

    this.muteButton!.addEventListener("click", () => {
      this.muted = !this.muted;
      this.dispatchEvent(new CustomEvent("gisst-mute"));
    });

    zoomButton!.addEventListener("click", () => {
      this.dispatchEvent(new CustomEvent("gisst-zoom", { detail: this.zoom }));
    });

    saveButton!.addEventListener("click", () => {
      this.dispatchEvent(new CustomEvent("gisst-save-state"));
    });

    recordButton!.addEventListener("click", () => {
      this.dispatchEvent(new CustomEvent("gisst-start-performance"));
    });

    stopButton!.addEventListener("click", () => {
      this.dispatchEvent(new CustomEvent("gisst-finish-performance"));
    });
  }

  get muted() {
    return this.mutedValue;
  }

  set muted(value: boolean) {
    this.mutedValue = value;
    if (this.muteButton) {
      this.muteButton.textContent = value ? "Unmute" : "Mute";
    }
  }

  get zoom() {
    return this.zoomValue;
  }

  set zoom(value: string) {
    this.zoomValue = value;
  }
}

customElements.define("editor-controls", EditorControls);
