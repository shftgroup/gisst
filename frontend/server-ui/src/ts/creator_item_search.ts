import * as scs from "./search_components";
import { GISSTItemSearch } from "./item_search";
import { type CreatorRecord, type InstanceRecord } from "./search_components";
import {
  type TemplateWithBindEvent as Template,
  type Hit,
} from "instantsearch.js";

function assert_creator_record(
  hit: Hit<InstanceRecord>,
): asserts hit is Hit<CreatorRecord> {
  if (!("creator_id" in hit || "hidden" in hit)) {
    throw new Error("Invalid hit for created item search results", {
      cause: hit,
    });
  }
}

export abstract class GISSTCreatorItemSearch extends GISSTItemSearch {
  limit_to_instance: string = "";
  limit_to_creator: string = "";
  show_creator_info: boolean = false;
  show_instance_info: boolean = false;
  show_hidden_state: boolean = false;
  constructor() {
    super();
  }
  abstract short_title_key(): string;
  abstract description_key(): string;
  screenshot(_hit: CreatorRecord): string | null {
    return null;
  }
  can_clone(_hit: CreatorRecord): boolean {
    return false;
  }
  item_template(): Template<Hit<InstanceRecord>> {
    return (hit: Hit<InstanceRecord>, { html, components }) => {
      assert_creator_record(hit);
      return html`
        <div
          class="${scs.hit_classes(hit, this.show_creator_info, this.show_instance_info)}"
        >
          ${scs.created_item_info(this.base_url, this.modeltype(), this.item_id(hit), hit, this.short_title_key(), this.description_key(), this.screenshot(hit), html, components)}
          ${this.show_instance_info ? scs.instance_info(this.base_url, hit, html, components) : html``}
          ${this.show_creator_info ? scs.creator_info(this.base_url, hit, html, components) : html``}
          ${this.show_hidden_state ? scs.hidden_state(this.item_id(hit), hit.hidden, html) : html``}
          <div class="gisst-Search-cell gisst-Search-actions-cell">
            ${this.show_hidden_state ? scs.hideshow_action(this.item_id(hit), hit.hidden, html) : html``}
            ${scs.play_actions(this.base_url, hit.instance_id, this.item_id(hit), this.modeltype(), this.can_clone(hit), html)}
          </div>
        </div>
      `;
    };
  }
  filters(): string[] {
    const filters = [];
    if (this.limit_to_instance != "") {
      filters.push(`instance_id = "${this.limit_to_instance}"`);
    }
    if (this.limit_to_creator != "") {
      filters.push(`creator_id = "${this.limit_to_creator}"`);
    }
    if (!this.show_hidden_state) {
      filters.push("hidden = false");
    }
    return filters;
  }
  read_attributes(): void {
    super.read_attributes();
    this.limit_to_instance = this.getAttribute("instance-id") ?? "";
    this.limit_to_creator = this.getAttribute("creator-id") ?? "";
    this.show_creator_info =
      (this.getAttribute("creator-info") ?? "true") == "true";
    this.show_hidden_state =
      this.active_creator_role <= 10 ||
      (this.limit_to_creator == this.active_creator_id &&
        this.active_creator_id != "");
    this.show_instance_info =
      (this.getAttribute("instance-info") ?? "true") == "true";
  }
  create_structure(): void {
    super.create_structure();
    const search_container = this.firstElementChild as HTMLElement;
    const results_body = search_container.querySelector<HTMLElement>(
      ".gisst-Search-results-body",
    )!;
    const results_header = search_container.querySelector(
      ".gisst-Search-results-header",
    )!;
    if (!this.show_hidden_state) {
      search_container.querySelector(".gisst-Search-hidestate")!.remove();
      results_header.classList.add("gisst-Search-no-hidestate");
    }
    if (!this.show_instance_info) {
      search_container.querySelector(".gisst-Search-instance-info")!.remove();
      results_header.classList.add("gisst-Search-no-instance");
    }
    if (!this.show_creator_info) {
      search_container.querySelector(".gisst-Search-creator-info")!.remove();
      results_header.classList.add("gisst-Search-no-creator");
    }
    results_body.addEventListener(
      "click",
      scs.hideshow_click_listener(this.base_url, `${this.modeltype()}s`),
    );
  }
}
