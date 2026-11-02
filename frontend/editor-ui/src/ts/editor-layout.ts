import html from "../html/editor-layout.html?raw";
import { GisstElement } from "./globals";

class EditorLayout extends GisstElement {
  connectedCallback() {
    if (this.childElementCount > 0) {
      return;
    }

    const template = document.createElement("template");
    template.innerHTML = html;
    this.append(template.content.cloneNode(true));
  }
}

customElements.define("editor-layout", EditorLayout);
