import * as widgets from "instantsearch.js/es/widgets";
import { type InstanceRecord } from "./search_components";
import { SearchOptions, SearchObject } from "./search_impl";
import {
  type TemplateWithBindEvent as Template,
  type Hit,
} from "instantsearch.js";
import { elementFromTemplate } from "./templates";

export type SearchFunction = (
  host: string,
  key: string,
  params: SearchOptions,
) => SearchObject;

export abstract class GISSTItemSearch extends HTMLElement {
  search_url: string = "";
  search_key: string = "";
  base_url: string = "";
  active_creator_id: string = "";
  active_creator_role: number = 0;
  // Will be initialized by connectedCallback
  search!: SearchObject;
  constructor() {
    super();
  }
  abstract modeltype(): string;
  abstract search_function(): SearchFunction;
  filters(): string[] {
    return [];
  }
  abstract item_id(hit: InstanceRecord): string;
  abstract short_title_key(): string;
  abstract description_key(): string;
  screenshot(_hit: InstanceRecord): string | null {
    return null;
  }
  can_clone(_hit: InstanceRecord): boolean {
    return false;
  }
  abstract item_template(): Template<Hit<InstanceRecord>>;
  read_attributes(): void {
    this.search_url = this.getAttribute("search-url") ?? "";
    this.search_key = this.getAttribute("search-key") ?? "";
    this.base_url = this.getAttribute("base-url") ?? "";
    if (this.search_url == "" || this.search_key == "") {
      throw new Error(
        `Cannot create ${this.modeltype()} search UI without search url and search key`,
      );
    }
    this.active_creator_id = this.getAttribute("user-creator-id") ?? "";
    this.active_creator_role = parseInt(
      this.getAttribute("user-role") ?? "100",
    );
  }
  create_structure(): void {
    const search_container = elementFromTemplate<HTMLDivElement>(
      `${this.modeltype()}-search`,
    );
    this.appendChild(search_container);
  }
  connectedCallback() {
    this.read_attributes();
    this.search = this.search_function()(this.search_url, this.search_key, {});
    this.classList.add(`gisst-${this.modeltype()}-search`);
    this.create_structure();
    const search_box = this.querySelector<HTMLElement>(".gisst-Search-box")!;
    const pagination = this.querySelector<HTMLElement>(
      ".gisst-Search-pagination",
    )!;
    const results = this.querySelector<HTMLElement>(
      ".gisst-Search-results-body",
    )!;
    this.search.addWidgets([
      widgets.searchBox({
        container: search_box,
      }),
      widgets.configure({
        hitsPerPage: 10,
        filters: this.filters().join(" AND "),
      }),
      widgets.pagination({
        container: pagination,
        padding: 0,
        showFirst: false,
        showLast: false,
      }),
      widgets.hits({
        container: results,
        templates: {
          item: this.item_template(),
        },
      }),
    ]);
    this.search.start();
  }
}
