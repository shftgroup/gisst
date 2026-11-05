import html from "../html/editor-sidebar.html?raw";
import { GisstElement } from "./globals";

class EditorSidebar extends GisstElement {
  connectedCallback() {
    if (this.querySelector(".editor-sidebar-column")) {
      return;
    }

    const template = document.createElement("template");
    template.innerHTML = html;
    this.append(template.content.cloneNode(true));

    this.querySelector("[data-action='instances']")!.addEventListener(
      "click",
      () => {
        this.dispatchEvent(new CustomEvent("gisst-instances"));
      },
    );

    this.querySelector("[data-action='add-work']")!.addEventListener(
      "click",
      () => {
        this.dispatchEvent(new CustomEvent("gisst-add-work"));
      },
    );
  }
}

customElements.define("editor-sidebar", EditorSidebar);
