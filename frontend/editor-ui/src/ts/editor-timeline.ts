import html from "../html/editor-timeline.html?raw";
import { GisstElement } from "./globals";

class EditorTimeline extends GisstElement {
  private filterValue = "";
  private selectedValue = "";

  connectedCallback() {
    if (this.querySelector(".editor-timeline-row")) {
      return;
    }

    const template = document.createElement("template");
    template.innerHTML = html;
    this.append(template.content.cloneNode(true));

    const empty = this.querySelector(".editor-timeline-empty");
    const card = this.querySelector(
      "editor-state-card, editor-save-card, editor-replay-card",
    );
    if (empty && card) {
      empty.remove();
    }
  }

  get filter() {
    return this.filterValue;
  }

  set filter(value: string) {
    this.filterValue = value;
  }

  get selected() {
    return this.selectedValue;
  }

  set selected(value: string) {
    this.selectedValue = value;
  }
}

customElements.define("editor-timeline", EditorTimeline);
