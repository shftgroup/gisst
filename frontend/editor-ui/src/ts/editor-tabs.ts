import html from "../html/editor-tabs.html?raw";
import { GisstElement } from "./globals";

class EditorTabs extends GisstElement {
  connectedCallback() {
    if (this.querySelector(".editor-tabs-bar")) {
      return;
    }

    const template = document.createElement("template");
    template.innerHTML = html;
    this.append(template.content.cloneNode(true));

    const filters = this.querySelector(".editor-tabs-filters")!;
    const buttons = this.querySelectorAll("button");
    buttons.forEach((button) => {
      button.addEventListener("click", () => {
        const tabId = button.getAttribute("data-tab") ?? "";
        if (filters.contains(button)) {
          filters.querySelectorAll("button").forEach((filterButton) => {
            filterButton.classList.remove("is-active");
          });
          button.classList.add("is-active");
        }

        this.dispatchEvent(new CustomEvent("gisst-tab", { detail: { tabId } }));
      });
    });
  }
}

customElements.define("editor-tabs", EditorTabs);
