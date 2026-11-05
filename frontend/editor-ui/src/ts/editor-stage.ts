import html from "../html/editor-stage.html?raw";
import { GisstElement } from "./globals";

class EditorStage extends GisstElement {
  connectedCallback() {
    if (!this.hasAttribute("mode")) {
      this.setAttribute("mode", "inactive");
    }

    if (this.querySelector(".editor-stage-label")) {
      return;
    }

    const template = document.createElement("template");
    template.innerHTML = html;
    this.append(template.content.cloneNode(true));
  }

  get mode() {
    return this.getAttribute("mode") ?? "inactive";
  }

  set mode(value: string) {
    this.setAttribute("mode", value);
  }
}

customElements.define("editor-stage", EditorStage);
