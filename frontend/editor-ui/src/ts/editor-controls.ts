import html from "../html/editor-controls.html?raw";
import muteIcon from "../img/editor-mute.svg";
import recordIcon from "../img/editor-record.svg";
import recordingIcon from "../img/editor-recording.svg";
import saveStateIcon from "../img/editor-save-state.svg";
import stopRecordIcon from "../img/editor-stop-record.svg";
import volumeIcon from "../img/editor-volume.svg";
import { GisstElement } from "./globals";

class EditorControls extends GisstElement {
  private muteButton!: HTMLButtonElement;
  private zoomButton!: HTMLButtonElement;
  private recordButton!: HTMLButtonElement;
  private menu!: HTMLElement;
  private mutedValue = false;
  private zoomValue = "1";

  connectedCallback() {
    if (this.querySelector(".editor-controls-row")) {
      return;
    }

    const template = document.createElement("template");
    template.innerHTML = html;
    this.append(template.content.cloneNode(true));

    this.muteButton = this.querySelector<HTMLButtonElement>(
      "#editor-controls-mute",
    )!;
    this.zoomButton = this.querySelector<HTMLButtonElement>(
      "#editor-controls-zoom",
    )!;
    this.recordButton = this.querySelector<HTMLButtonElement>(
      "#editor-controls-record",
    )!;
    this.menu = this.querySelector<HTMLElement>(".editor-controls-menu")!;
    const saveButton = this.querySelector<HTMLButtonElement>(
      "#editor-controls-save",
    )!;
    const stopButton = this.querySelector<HTMLButtonElement>(
      "#editor-controls-stop",
    )!;
    const fullscreenButton = this.querySelector<HTMLButtonElement>(
      "#editor-controls-fullscreen",
    )!;
    const sizeControl = this.querySelector(".editor-controls-size")!;

    saveButton.querySelector("img")!.src = saveStateIcon;
    this.recordButton.querySelector("img")!.src = recordIcon;
    stopButton.querySelector("img")!.src = stopRecordIcon;

    this.muted = this.mutedValue;
    this.zoom = this.zoomValue;

    this.muteButton.addEventListener("click", () => {
      this.muted = !this.muted;
      this.setActive(this.muteButton);
      this.dispatchEvent(new CustomEvent("gisst-mute"));
    });

    this.zoomButton.addEventListener("click", () => {
      this.setActive(this.zoomButton);
      // `hidden` is boolean, or the string "until-found". Any value other than false means the menu is not open.
      this.setMenuOpen(this.menu.hidden !== false);
    });

    this.querySelectorAll("[data-zoom]").forEach((item) => {
      item.addEventListener("click", () => {
        this.zoom = item.getAttribute("data-zoom") ?? "1";
        this.setMenuOpen(false);
        this.setActive(this.zoomButton);
        this.dispatchEvent(
          new CustomEvent("gisst-zoom", { detail: this.zoom }),
        );
      });
    });

    fullscreenButton.addEventListener("click", () => {
      this.setMenuOpen(false);
      this.setActive(this.zoomButton);
      this.dispatchEvent(new CustomEvent("gisst-fullscreen"));
    });

    saveButton.addEventListener("click", () => {
      this.setActive(saveButton);
      this.dispatchEvent(new CustomEvent("gisst-save-state"));
    });

    this.recordButton.addEventListener("click", () => {
      this.setActive(this.recordButton);
      this.dispatchEvent(new CustomEvent("gisst-start-performance"));
    });

    stopButton.addEventListener("click", () => {
      this.setActive(stopButton);
      this.dispatchEvent(new CustomEvent("gisst-finish-performance"));
    });

    document.addEventListener("click", (event) => {
      if (!sizeControl.contains(event.target as Node)) {
        this.setMenuOpen(false);
      }
    });
  }

  private setMenuOpen(open: boolean) {
    this.menu.hidden = !open;
    this.zoomButton.setAttribute("aria-expanded", String(open));
  }

  private setActive(button: HTMLButtonElement) {
    this.querySelectorAll(".editor-controls-square").forEach((square) => {
      square.classList.remove("is-active");
    });
    button.classList.add("is-active");
    this.recordButton.querySelector("img")!.src =
      button === this.recordButton ? recordingIcon : recordIcon;
  }

  get muted() {
    return this.mutedValue;
  }

  set muted(value: boolean) {
    this.mutedValue = value;
    this.muteButton.setAttribute("aria-label", value ? "Unmute" : "Mute");
    this.muteButton.querySelector("img")!.src = value ? muteIcon : volumeIcon;
  }

  get zoom() {
    return this.zoomValue;
  }

  set zoom(value: string) {
    this.zoomValue = value;
    this.zoomButton.textContent = value === "Fit" ? "Fit" : `${value}x`;
    this.querySelectorAll("[data-zoom]").forEach((item) => {
      item.classList.toggle(
        "is-current",
        item.getAttribute("data-zoom") === value,
      );
    });
  }
}

customElements.define("editor-controls", EditorControls);
