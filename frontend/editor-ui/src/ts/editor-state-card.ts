import html from "../html/editor-state-card.html?raw";
import { GisstElement } from "./globals";

class EditorStateCard extends GisstElement {
  connectedCallback() {
    if (this.querySelector(".editor-card")) {
      return;
    }

    const template = document.createElement("template");
    template.innerHTML = html;
    this.append(template.content.cloneNode(true));

    const card = this.querySelector(".editor-card")!;
    card.addEventListener("click", () => {
      this.dispatchEvent(new CustomEvent("gisst-load-state"));
    });

    this.querySelector("[data-action='upload']")!.addEventListener(
      "click",
      (event) => {
        event.stopPropagation();
        this.dispatchEvent(new CustomEvent("gisst-upload"));
      },
    );

    this.querySelector("[data-action='download']")!.addEventListener(
      "click",
      (event) => {
        event.stopPropagation();
        this.dispatchEvent(new CustomEvent("gisst-download"));
      },
    );

    this.querySelector("[data-action='edit']")!.addEventListener(
      "click",
      (event) => {
        event.stopPropagation();
        this.dispatchEvent(new CustomEvent("gisst-edit"));
      },
    );
  }
}

customElements.define("editor-state-card", EditorStateCard);
