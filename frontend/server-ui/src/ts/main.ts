import {
  instantMeiliSearch,
  InstantMeiliSearchOptions,
} from "@meilisearch/instant-meilisearch";
import { default as instantsearch, InstantSearch } from "instantsearch.js";
import { default as SparkMD5 } from "spark-md5";
import "instantsearch.css/themes/reset.css";
import "gisst-style/base.css";
import "../css/server-ui-main.css";
import "../css/server-ui-new-instance.css";
import "../css/server-ui-search.css";
import "../css/server-ui-instance.css";
export type SearchOptions = InstantMeiliSearchOptions;
export * as search from "instantsearch.js";
import * as widgets from "instantsearch.js/es/widgets";
export * as widgets from "instantsearch.js/es/widgets";
import * as tus from "tus-js-client";
import templates from "../html/templates.html?raw";
import connectStats from "instantsearch.js/es/connectors/stats/connectStats";
import { V86Image, V86ImageType, V86Options } from "./v86";

// Import all templates from templates.html
const TEMPLATES = document.createElement("template");
TEMPLATES.innerHTML = templates.trim();

/**
 * Create a new instance of an element from a template.
 * @param template_id ID of the template to instantiate.
 * @returns a {@link DocumentFragment} of the template being imported.
 */
function elementFromTemplates(template_id: string): DocumentFragment {
  const template_element = TEMPLATES.content.querySelector(
    "#" + template_id,
  ) as HTMLTemplateElement;
  return document.importNode(template_element.content, true);
}

function search_instances(
  search_host: string,
  search_key: string,
  params: SearchOptions,
): InstantSearch {
  params.primaryKey = "instance_id";
  const { searchClient } = instantMeiliSearch(search_host, search_key, params);
  return instantsearch({
    indexName: "instance",
    searchClient,
  });
}

export function search_states(
  search_host: string,
  search_key: string,
  params: SearchOptions,
): InstantSearch {
  params.primaryKey = "state_id";
  const { searchClient } = instantMeiliSearch(search_host, search_key, params);
  return instantsearch({
    indexName: "state",
    searchClient,
  });
}

export function search_saves(
  search_host: string,
  search_key: string,
  params: SearchOptions,
): InstantSearch {
  params.primaryKey = "save_id";
  const { searchClient } = instantMeiliSearch(search_host, search_key, params);
  return instantsearch({
    indexName: "save",
    searchClient,
  });
}

export function search_replays(
  search_host: string,
  search_key: string,
  params: SearchOptions,
): InstantSearch {
  params.primaryKey = "replay_id";
  const { searchClient } = instantMeiliSearch(search_host, search_key, params);
  return instantsearch({
    indexName: "replay",
    searchClient,
  });
}

// TODO: create wrappers for these four element types using a single div, string template, innerHTML like frontend-embed?

class GISSTInstanceSearch extends HTMLElement {
  constructor() {
    super();
  }
  connectedCallback() {
    const search_url = this.getAttribute("search-url");
    const search_key = this.getAttribute("search-key");
    const base_url = this.getAttribute("base-url");
    if (
      search_url == undefined ||
      search_key == undefined ||
      base_url == undefined
    ) {
      throw new Error(
        "Cannot create instance search UI without search url, search key, and base url",
      );
    }
    this.classList.add("gisst-instance-search");

    const search_container = document.createElement("div");
    search_container.setAttribute("class", "gisst-Search-container");

    const search_box = document.createElement("div");
    search_container.appendChild(search_box);

    const results_container = document.createElement("div");
    results_container.setAttribute("class", "gisst-Search-results-container");
    results_container.setAttribute("class", "gisst-Search-table-view");
    results_container.innerHTML = `
    <div class="gisst-Search-results-header">
      <div class="gisst-Search-header-cell gisst-Search-header-description">Description</div>
      <div class="gisst-Search-header-cell gisst-Search-header-platform">Platform</div>
      <div class="gisst-Search-header-cell gisst-Search-header-version">Version</div>
      <div class="gisst-Search-header-cell gisst-Search-header-actions">Actions</div>
    </div>
    `;

    const results_body = document.createElement("div");
    results_body.setAttribute("class", "gisst-Search-results-body");
    results_container.appendChild(results_body);
    search_container.appendChild(results_container);

    const pagination = document.createElement("div");
    search_container.appendChild(pagination);
    this.appendChild(search_container);

    const search = search_instances(search_url, search_key, {});
    search.addWidgets([
      widgets.searchBox({
        container: search_box,
      }),
      widgets.hits({
        container: results_body,
        templates: {
          item(hit, { html, components }) {
            return html`
              <div class="gisst-Search-results-row">
                <div class="gisst-Search-cell gisst-Search-instance-work-name">
                  <a href="${base_url}/instances/${hit.instance_id}"
                    >${components.Highlight({ hit, attribute: "work_name" })}</a
                  >
                </div>
                <div
                  class="gisst-Search-cell gisst-Search-instance-platform-info"
                >
                  ${components.Highlight({ hit, attribute: "work_platform" })}
                </div>
                <div
                  class="gisst-Search-cell gisst-Search-instance-version-info"
                >
                  ${hit.work_version}
                </div>
                <div class="gisst-Search-cell gisst-Search-actions-cell">
                  <a
                    class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only"
                    href="${base_url}/play/${hit.instance_id}"
                    >Play</a
                  >
                  <a
                    class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only"
                    href="${base_url}/play/${hit.instance_id}"
                    title="Play"
                  >
                    <svg
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    >
                      <polygon points="5 3 19 12 5 21 5 3"></polygon>
                    </svg>
                  </a>
                  <a
                    class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only"
                    href="${base_url}/data/${hit.instance_id}"
                    >Cite</a
                  >
                  <a
                    class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only"
                    href="${base_url}/data/${hit.instance_id}"
                    title="Cite"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                      <g transform="translate(-25.751 -58.132)">
                        <circle
                          cx="30.974"
                          cy="63.355"
                          r="5.2227"
                          stroke-width=".26458"
                        />
                        <path
                          d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176"
                          stroke-width=".27134"
                        />
                      </g>
                      <g transform="translate(-14.007 -58.132)">
                        <circle
                          cx="30.974"
                          cy="63.355"
                          r="5.2227"
                          stroke-width=".26458"
                        />
                        <path
                          d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176"
                          stroke-width=".27134"
                        />
                      </g>
                    </svg>
                  </a>
                </div>
              </div>
            `;
          },
        },
      }),
      widgets.configure({ hitsPerPage: 10 }),
      widgets.pagination({
        container: pagination,
        padding: 0,
        showFirst: false,
        showLast: false,
      }),
    ]);

    search.start();
  }
}

customElements.define("gisst-instance-search", GISSTInstanceSearch);
class GISSTStateSearch extends HTMLElement {
  constructor() {
    super();
  }
  connectedCallback() {
    const search_url = this.getAttribute("search-url");
    const search_key = this.getAttribute("search-key");
    const base_url = this.getAttribute("base-url");
    if (
      search_url == undefined ||
      search_key == undefined ||
      base_url == undefined
    ) {
      throw new Error(
        "Cannot create state search UI without search url, search key, and base url",
      );
    }
    const limit_to_instance = this.getAttribute("instance-id");
    const limit_to_creator = this.getAttribute("creator-id");
    const active_creator_id = this.getAttribute("user-creator-id") ?? "";
    const active_creator_role = parseInt(
      this.getAttribute("user-role") ?? "100",
    );
    const show_hidden_state =
      active_creator_role <= 10 ||
      (limit_to_creator == active_creator_id && active_creator_id != "");
    const show_creator_info =
      (this.getAttribute("creator-info") ?? "true") == "true";
    const show_instance_info =
      (this.getAttribute("instance-info") ?? "true") == "true";
    // TODO: Now that environment framework is part of the info, we don't need this attribute and instead can make it hit-by-hit
    const can_clone = (this.getAttribute("can-clone") ?? "false") === "true";
    const filters: string[] = [];
    if (limit_to_instance && limit_to_instance != "") {
      filters.push(`instance_id = "${limit_to_instance}"`);
    }
    if (limit_to_creator && limit_to_creator != "") {
      filters.push(`creator_id = "${limit_to_creator}"`);
    }
    if (!show_hidden_state) {
      filters.push("hidden = false");
    }
    this.classList.add("gisst-state-search");

    const search_container = document.createElement("div");
    search_container.setAttribute("class", "gisst-Search-container");
    this.appendChild(search_container);

    const search_box = document.createElement("div");
    search_container.appendChild(search_box);

    const results_container = document.createElement("div");
    results_container.classList.add("gisst-Search-results-container");
    results_container.classList.add("gisst-Search-table-view");
    results_container.innerHTML = `
      <div class="gisst-Search-results-header gisst-Search-responsive-header">
          <div class="gisst-Search-header-cell gisst-Search-screenshot-header">Preview</div>
          <div class="gisst-Search-header-cell gisst-Search-state-info">Description</div>
          <div class="gisst-Search-header-cell gisst-Search-instance-info">Instance</div>
          <div class="gisst-Search-header-cell gisst-Search-creator-info">Creator</div>
          ${show_hidden_state ? '<div class="gisst-Search-header-cell gisst-Search-hidestate">Hidden</div>' : ""}
          <div class="gisst-Search-header-cell gisst-Search-actions-cell">Actions</div>
      </div>
    `;
    if (!show_instance_info) {
      results_container.querySelector(".gisst-Search-instance-info")!.remove();
      results_container
        .querySelector(".gisst-Search-results-header")!
        .classList.add("gisst-Search-no-instance");
    }
    if (!show_creator_info) {
      results_container.querySelector(".gisst-Search-creator-info")!.remove();
      results_container
        .querySelector(".gisst-Search-results-header")!
        .classList.add("gisst-Search-no-creator");
    }

    const results_body = document.createElement("div");
    results_body.setAttribute("class", "gisst-Search-results-body");
    results_container.appendChild(results_body);
    search_container.appendChild(results_container);

    const pagination = document.createElement("div");
    search_container.appendChild(pagination);

    const search = search_states(search_url, search_key, {});
    search.addWidgets([
      widgets.searchBox({
        container: search_box,
      }),
      widgets.configure({ hitsPerPage: 10, filters: filters.join(" AND ") }),
      widgets.pagination({
        container: pagination,
        padding: 0,
        showFirst: false,
        showLast: false,
      }),
      /* TODO: these nested templates are gnarly, what can be done? */
      widgets.hits({
        container: results_body,
        templates: {
          item: (hit, { html, components }) => html`
            <div
              class="gisst-Search-results-row gisst-Search-responsive-results-row
           ${!show_creator_info ? "gisst-Search-no-creator" : ""}
           ${hit.hidden ? "gisst-Search-item-hidden" : ""}
           ${!show_instance_info ? "gisst-Search-no-instance" : ""}"
            >
              <div class="gisst-Search-cell gisst-Search-screenshot-cell">
                <img
                  class="gisst-Search-screenshot"
                  src="data:image/png;base64,${hit.screenshot_data}"
                  alt="${hit.state_description} from instance ${hit.work_name}"
                />
              </div>
              <div class="gisst-Search-cell gisst-Search-state-info">
                <div class="gisst-Search-name">
                  <a
                    href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}"
                    >${components.Highlight({
                      hit,
                      attribute: "state_name",
                    })}</a
                  >
                </div>
                <div class="gisst-Search-description">
                  ${components.Highlight({
                    hit,
                    attribute: "state_description",
                  })}
                </div>
              </div>
              ${show_instance_info
                ? html`
                    <div class="gisst-Search-cell gisst-Search-instance-info">
                      <div class="gisst-Search-instance-name">
                        <a href="${base_url}/instances/${hit.instance_id}"
                          >${components.Highlight({
                            hit,
                            attribute: "work_name",
                          })}</a
                        >
                      </div>
                      <div class="gisst-Search-instance-details">
                        ${hit.work_version} • ${hit.work_platform}
                      </div>
                    </div>
                  `
                : ""}
              ${show_creator_info
                ? html`
                    <div class="gisst-Search-cell gisst-Search-creator-info">
                      <a href="${base_url}/creators/${hit.creator_id}"
                        >${components.Highlight({
                          hit,
                          attribute: "creator_full_name",
                        })}</a
                      >
                    </div>
                  `
                : ""}
              ${show_hidden_state
                ? html`
                <div class="gisst-Search-cell gisst-Search-hidden-info">
                <input name="hidden-${hit.state_id}" class="hide-show-checkbox" type="checkbox" data-id="${hit.state_id}" checked="${hit.hidden}"></input>
                </div>
                `
                : ""}
              <div class="gisst-Search-cell gisst-Search-actions-cell">
                <a
                  class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only"
                  href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}"
                  >Play</a
                >
                <a
                  class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only"
                  href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}"
                  title="Play"
                >
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <polygon points="5 3 19 12 5 21 5 3" />
                  </svg>
                </a>
                <a
                  class="gisst-Search-btn gisst-Search-btn-secondary gisst-Search-btn-text-only"
                  href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}&boot_into_record=true"
                  >Record</a
                >
                <a
                  class="gisst-Search-btn gisst-Search-btn-secondary gisst-Search-btn-icon gisst-Search-btn-icon-only"
                  href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}&boot_into_record=true"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <circle cx="12" cy="12" r="10" />
                    <circle cx="12" cy="12" r="4" fill="currentColor" />
                  </svg>
                </a>
                <a
                  class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only"
                  href="${base_url}/data/${hit.instance_id}?state=${hit.state_id}"
                  >Cite</a
                >
                <a
                  class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only"
                  href="${base_url}/data/${hit.instance_id}?state=${hit.state_id}"
                  title="Cite"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                    <g transform="translate(-25.751 -58.132)">
                      <circle
                        cx="30.974"
                        cy="63.355"
                        r="5.2227"
                        stroke-width=".26458"
                      />
                      <path
                        d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176"
                        stroke-width=".27134"
                      />
                    </g>
                    <g transform="translate(-14.007 -58.132)">
                      <circle
                        cx="30.974"
                        cy="63.355"
                        r="5.2227"
                        stroke-width=".26458"
                      />
                      <path
                        d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176"
                        stroke-width=".27134"
                      />
                    </g>
                  </svg>
                </a>
                ${can_clone
                  ? html`
                      <a
                        class="gisst-Search-btn gisst-Search-btn-accent gisst-Search-btn-text-only"
                        href="${hit.instance_id}/clone?state=${hit.state_id}"
                        >Clone</a
                      >
                      <a
                        class="gisst-Search-btn gisst-Search-btn-accent gisst-Search-btn-icon gisst-Search-btn-icon-only"
                        href="${hit.instance_id}/clone?state=${hit.state_id}"
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                        >
                          <rect
                            x="9"
                            y="9"
                            width="13"
                            height="13"
                            rx="2"
                            ry="2"
                          />
                          <path
                            d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"
                          />
                        </svg>
                      </a>
                    `
                  : ""}
              </div>
            </div>
          `,
        },
      }),
    ]);

    results_body.addEventListener("click", function (event) {
      void (async () => {
        if (
          event.target &&
          (event.target as HTMLElement).classList.contains("hide-show-checkbox")
        ) {
          const checkbox = event.target as HTMLInputElement;
          const id = checkbox.dataset["id"];
          try {
            await (
              await fetch(`${base_url}/states/${id}/hide`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ state: checkbox.checked }),
              })
            ).text();
            if (checkbox.checked) {
              checkbox.parentElement?.parentElement?.classList.add(
                "gisst-Search-item-hidden",
              );
            } else {
              checkbox.parentElement?.parentElement?.classList.remove(
                "gisst-Search-item-hidden",
              );
            }
          } catch (error) {
            console.error(error);
            checkbox.checked = !checkbox.checked;
          }
        }
      })();
    });
    search.start();
  }
}

customElements.define("gisst-state-search", GISSTStateSearch);

class GISSTSaveSearch extends HTMLElement {
  constructor() {
    super();
  }
  connectedCallback() {
    const search_url = this.getAttribute("search-url");
    const search_key = this.getAttribute("search-key");
    const base_url = this.getAttribute("base-url");
    if (
      search_url == undefined ||
      search_key == undefined ||
      base_url == undefined
    ) {
      throw new Error(
        "Cannot create save search UI without search url, search key, and base url",
      );
    }
    const limit_to_instance = this.getAttribute("instance-id");
    const limit_to_creator = this.getAttribute("creator-id");
    const active_creator_id = this.getAttribute("user-creator-id") ?? "";
    const active_creator_role = parseInt(this.getAttribute("user-role") ?? "0");
    const show_creator_info =
      (this.getAttribute("creator-info") ?? "true") == "true";
    const show_hidden_state =
      active_creator_role <= 10 ||
      (limit_to_creator == active_creator_id && active_creator_id != "");
    const show_instance_info =
      (this.getAttribute("instance-info") ?? "true") == "true";
    const filters: string[] = [];
    if (limit_to_instance && limit_to_instance != "") {
      filters.push(`instance_id = "${limit_to_instance}"`);
    }
    if (limit_to_creator && limit_to_creator != "") {
      filters.push(`creator_id = "${limit_to_creator}"`);
    }
    if (!show_hidden_state) {
      filters.push("hidden = false");
    }
    this.classList.add("gisst-save-search");
    const search_container = document.createElement("div");
    search_container.setAttribute("class", "gisst-Search-container");
    this.appendChild(search_container);

    const search_box = document.createElement("div");
    search_container.appendChild(search_box);

    const results_container = document.createElement("div");
    results_container.classList.add("gisst-Search-results-container");
    results_container.classList.add("gisst-Search-table-view");
    results_container.innerHTML = `
      <div class="gisst-Search-results-header gisst-Search-responsive-header">
          <div class="gisst-Search-header-cell gisst-Search-save-info">Description</div>
          <div class="gisst-Search-header-cell gisst-Search-instance-info">Instance</div>
          <div class="gisst-Search-header-cell gisst-Search-creator-info">Creator</div>
          ${show_hidden_state ? '<div class="gisst-Search-header-cell gisst-Search-hidestate">Hidden</div>' : ""}
          <div class="gisst-Search-header-cell gisst-Search-actions-cell">Actions</div>
      </div>
    `;
    if (!show_instance_info) {
      results_container.querySelector(".gisst-Search-instance-info")!.remove();
      results_container
        .querySelector(".gisst-Search-results-header")!
        .classList.add("gisst-Search-no-instance");
    }
    if (!show_creator_info) {
      results_container.querySelector(".gisst-Search-creator-info")!.remove();
      results_container
        .querySelector(".gisst-Search-results-header")!
        .classList.add("gisst-Search-no-creator");
    }

    const results_body = document.createElement("div");
    results_body.setAttribute("class", "gisst-Search-results-body");
    results_container.appendChild(results_body);
    search_container.appendChild(results_container);

    const pagination = document.createElement("div");
    search_container.appendChild(pagination);

    const search = search_saves(search_url, search_key, {});
    search.addWidgets([
      widgets.searchBox({
        container: search_box,
      }),
      widgets.configure({ hitsPerPage: 10, filters: filters.join(" AND ") }),
      widgets.pagination({
        container: pagination,
        padding: 0,
        showFirst: false,
        showLast: false,
      }),
      /* TODO: these nested templates are gnarly, what can be done? */
      widgets.hits({
        container: results_body,
        templates: {
          item: (hit, { html, components }) => html`
            <div
              class="gisst-Search-results-row gisst-Search-responsive-results-row
           ${!show_creator_info ? "gisst-Search-no-creator" : ""}
           ${hit.hidden ? "gisst-Search-item-hidden" : ""}
           ${!show_instance_info ? "gisst-Search-no-instance" : ""}"
            >
              <div class="gisst-Search-cell gisst-Search-save-info">
                <div class="gisst-Search-name">
                  <a
                    href="${base_url}/play/${hit.instance_id}?save=${hit.save_id}"
                    >${components.Highlight({
                      hit,
                      attribute: "save_short_desc",
                    })}</a
                  >
                </div>
                <div class="gisst-Search-description">
                  ${components.Highlight({
                    hit,
                    attribute: "save_description",
                  })}
                </div>
              </div>
              ${show_instance_info
                ? html`
                    <div class="gisst-Search-cell gisst-Search-instance-info">
                      <div class="gisst-Search-instance-name">
                        <a href="${base_url}/instances/${hit.instance_id}"
                          >${components.Highlight({
                            hit,
                            attribute: "work_name",
                          })}</a
                        >
                      </div>
                      <div class="gisst-Search-instance-details">
                        ${hit.work_version} • ${hit.work_platform}
                      </div>
                    </div>
                  `
                : ""}
              ${show_creator_info
                ? html`
                    <div class="gisst-Search-cell gisst-Search-creator-info">
                      <a href="${base_url}/creators/${hit.creator_id}"
                        >${components.Highlight({
                          hit,
                          attribute: "creator_full_name",
                        })}</a
                      >
                    </div>
                  `
                : ""}
              ${show_hidden_state
                ? html`
                <div class="gisst-Search-cell gisst-Search-hidden-info">
                <input name="hidden-${hit.state_id}" class="hide-show-checkbox" type="checkbox" data-id="${hit.state_id}" checked="${hit.hidden}"></input>
                </div>
                `
                : ""}
              <div class="gisst-Search-cell gisst-Search-actions-cell">
                ${show_hidden_state
                  ? html`
                <label for="hidden-${hit.save_id}" class="gisst-Search-checkbox-label">Hide?</label>
                <input name="hidden-${hit.save_id}" class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only hide-show-checkbox" type="checkbox" data-id="${hit.save_id}" checked="${hit.hidden}"></input>
                `
                  : ""}
                <a
                  class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only"
                  href="${base_url}/play/${hit.instance_id}?save=${hit.save_id}"
                  >Play</a
                >
                <a
                  class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only"
                  href="${base_url}/play/${hit.instance_id}?save=${hit.save_id}"
                  title="Play"
                >
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <polygon points="5 3 19 12 5 21 5 3" />
                  </svg>
                </a>
                <a
                  class="gisst-Search-btn gisst-Search-btn-secondary gisst-Search-btn-text-only"
                  href="${base_url}/play/${hit.instance_id}?save=${hit.save_id}&boot_into_record=true"
                  >Record</a
                >
                <a
                  class="gisst-Search-btn gisst-Search-btn-secondary gisst-Search-btn-icon gisst-Search-btn-icon-only"
                  href="${base_url}/play/${hit.instance_id}?save=${hit.save_id}&boot_into_record=true"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <circle cx="12" cy="12" r="10" />
                    <circle cx="12" cy="12" r="4" fill="currentColor" />
                  </svg>
                </a>

                <a
                  class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only"
                  href="${base_url}/data/${hit.instance_id}?save=${hit.save_id}"
                  >Cite</a
                >
                <a
                  class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only"
                  href="${base_url}/data/${hit.instance_id}?save=${hit.save_id}"
                  title="Cite"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                    <g transform="translate(-25.751 -58.132)">
                      <circle
                        cx="30.974"
                        cy="63.355"
                        r="5.2227"
                        stroke-width=".26458"
                      />
                      <path
                        d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176"
                        stroke-width=".27134"
                      />
                    </g>
                    <g transform="translate(-14.007 -58.132)">
                      <circle
                        cx="30.974"
                        cy="63.355"
                        r="5.2227"
                        stroke-width=".26458"
                      />
                      <path
                        d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176"
                        stroke-width=".27134"
                      />
                    </g>
                  </svg>
                </a>
              </div>
            </div>
          `,
        },
      }),
    ]);
    results_body.addEventListener("click", function (event) {
      void (async () => {
        if (
          event.target &&
          (event.target as HTMLElement).classList.contains("hide-show-checkbox")
        ) {
          const checkbox = event.target as HTMLInputElement;
          const id = checkbox.dataset["id"];
          try {
            await (
              await fetch(`${base_url}/saves/${id}/hide`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ state: checkbox.checked }),
              })
            ).text();
            if (checkbox.checked) {
              checkbox.parentElement?.parentElement?.classList.add(
                "gisst-Search-item-hidden",
              );
            } else {
              checkbox.parentElement?.parentElement?.classList.remove(
                "gisst-Search-item-hidden",
              );
            }
          } catch (error) {
            console.error(error);
            checkbox.checked = !checkbox.checked;
          }
        }
      })();
    });
    search.start();
  }
}

customElements.define("gisst-save-search", GISSTSaveSearch);

class GISSTPerformanceSearch extends HTMLElement {
  constructor() {
    super();
  }
  connectedCallback() {
    const search_url = this.getAttribute("search-url");
    const search_key = this.getAttribute("search-key");
    const base_url = this.getAttribute("base-url");
    if (
      search_url == undefined ||
      search_key == undefined ||
      base_url == undefined
    ) {
      throw new Error(
        "Cannot create performance search UI without search url, search key, and base url",
      );
    }
    const limit_to_instance = this.getAttribute("instance-id");
    const limit_to_creator = this.getAttribute("creator-id");
    const active_creator_id = this.getAttribute("user-creator-id") ?? "";
    const active_creator_role = parseInt(this.getAttribute("user-role") ?? "0");
    const show_creator_info =
      (this.getAttribute("creator-info") ?? "true") == "true";
    const show_hidden_state =
      active_creator_role <= 10 ||
      (limit_to_creator == active_creator_id && active_creator_id != "");
    const show_instance_info =
      (this.getAttribute("instance-info") ?? "true") == "true";
    const filters: string[] = [];
    if (limit_to_instance && limit_to_instance != "") {
      filters.push(`instance_id = "${limit_to_instance}"`);
    }
    if (limit_to_creator && limit_to_creator != "") {
      filters.push(`creator_id = "${limit_to_creator}"`);
    }
    if (!show_hidden_state) {
      filters.push("hidden = false");
    }
    this.classList.add("gisst-performance-search");
    const search_container = document.createElement("div");
    search_container.setAttribute("class", "gisst-Search-container");
    this.appendChild(search_container);

    const search_box = document.createElement("div");
    search_container.appendChild(search_box);

    const results_container = document.createElement("div");
    results_container.classList.add("gisst-Search-results-container");
    results_container.classList.add("gisst-Search-table-view");
    results_container.innerHTML = `
      <div class="gisst-Search-results-header gisst-Search-responsive-header">
          <div class="gisst-Search-header-cell gisst-Search-performance-info">Description</div>
          <div class="gisst-Search-header-cell gisst-Search-instance-info">Instance</div>
          <div class="gisst-Search-header-cell gisst-Search-creator-info">Creator</div>
          ${show_hidden_state ? '<div class="gisst-Search-header-cell gisst-Search-hidestate">Hidden</div>' : ""}
          <div class="gisst-Search-header-cell gisst-Search-actions-cell">Actions</div>
      </div>
    `;
    if (!show_instance_info) {
      results_container.querySelector(".gisst-Search-instance-info")!.remove();
      results_container
        .querySelector(".gisst-Search-results-header")!
        .classList.add("gisst-Search-no-instance");
    }
    if (!show_creator_info) {
      results_container.querySelector(".gisst-Search-creator-info")!.remove();
      results_container
        .querySelector(".gisst-Search-results-header")!
        .classList.add("gisst-Search-no-creator");
    }

    const results_body = document.createElement("div");
    results_body.setAttribute("class", "gisst-Search-results-body");
    results_container.appendChild(results_body);
    search_container.appendChild(results_container);

    const pagination = document.createElement("div");
    search_container.appendChild(pagination);

    const search = search_replays(search_url, search_key, {});
    search.addWidgets([
      widgets.searchBox({
        container: search_box,
      }),
      widgets.configure({ hitsPerPage: 10, filters: filters.join(" AND ") }),
      widgets.pagination({
        container: pagination,
        padding: 0,
        showFirst: false,
        showLast: false,
      }),
      /* TODO: these nested templates are gnarly, what can be done? */
      widgets.hits({
        container: results_body,
        templates: {
          item: (hit, { html, components }) => html`
            <div
              class="gisst-Search-results-row gisst-Search-responsive-results-row
           ${!show_creator_info ? "gisst-Search-no-creator" : ""}
           ${hit.hidden ? "gisst-Search-item-hidden" : ""}
           ${!show_instance_info ? "gisst-Search-no-instance" : ""}"
            >
              <div class="gisst-Search-cell gisst-Search-performance-info">
                <div class="gisst-Search-name">
                  <a
                    href="${base_url}/play/${hit.instance_id}?replay=${hit.replay_id}"
                    >${components.Highlight({
                      hit,
                      attribute: "replay_name",
                    })}</a
                  >
                </div>
                <div class="gisst-Search-cell gisst-Search-description">
                  ${components.Highlight({
                    hit,
                    attribute: "replay_description",
                  })}
                </div>
              </div>
              ${show_instance_info
                ? html`
                    <div class="gisst-Search-cell gisst-Search-instance-info">
                      <div class="gisst-Search-instance-name">
                        <a href="${base_url}/instances/${hit.instance_id}"
                          >${components.Highlight({
                            hit,
                            attribute: "work_name",
                          })}</a
                        >
                      </div>
                      <div class="gisst-Search-instance-details">
                        ${hit.work_version} • ${hit.work_platform}
                      </div>
                    </div>
                  `
                : ""}
              ${show_creator_info
                ? html`
                    <div class="gisst-Search-cell gisst-Search-creator-info">
                      <a href="${base_url}/creators/${hit.creator_id}"
                        >${components.Highlight({
                          hit,
                          attribute: "creator_full_name",
                        })}</a
                      >
                    </div>
                  `
                : ""}
              ${show_hidden_state
                ? html`
                <div class="gisst-Search-cell gisst-Search-hidden-info">
                <input name="hidden-${hit.state_id}" class="hide-show-checkbox" type="checkbox" data-id="${hit.state_id}" checked="${hit.hidden}"></input>
                </div>
                `
                : ""}
              <div class="gisst-Search-cell gisst-Search-actions-cell">
                ${show_hidden_state
                  ? html`
                <label for="hidden-${hit.replay_id}" class="gisst-Search-checkbox-label">Hide?</label>
                <input name="hidden-${hit.replay_id}" class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only hide-show-checkbox" type="checkbox" data-id="${hit.replay_id}" checked="${hit.hidden}"></input>
                `
                  : ""}
                <a
                  class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only"
                  href="${base_url}/play/${hit.instance_id}?replay=${hit.replay_id}"
                  >Play</a
                >
                <a
                  class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only"
                  href="${base_url}/play/${hit.instance_id}?replay=${hit.replay_id}"
                  title="Play"
                >
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <polygon points="5 3 19 12 5 21 5 3" />
                  </svg>
                </a>
                <a
                  class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only"
                  href="${base_url}/data/${hit.instance_id}?replay=${hit.replay_id}"
                  >Cite</a
                >
                <a
                  class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only"
                  href="${base_url}/data/${hit.instance_id}?replay=${hit.replay_id}"
                  title="Cite"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                    <g transform="translate(-25.751 -58.132)">
                      <circle
                        cx="30.974"
                        cy="63.355"
                        r="5.2227"
                        stroke-width=".26458"
                      />
                      <path
                        d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176"
                        stroke-width=".27134"
                      />
                    </g>
                    <g transform="translate(-14.007 -58.132)">
                      <circle
                        cx="30.974"
                        cy="63.355"
                        r="5.2227"
                        stroke-width=".26458"
                      />
                      <path
                        d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176"
                        stroke-width=".27134"
                      />
                    </g>
                  </svg>
                </a>
              </div>
            </div>
          `,
        },
      }),
    ]);

    results_body.addEventListener("click", function (event) {
      void (async () => {
        if (
          event.target &&
          (event.target as HTMLElement).classList.contains("hide-show-checkbox")
        ) {
          const checkbox = event.target as HTMLInputElement;
          const id = checkbox.dataset["id"];
          try {
            await (
              await fetch(`${base_url}/replays/${id}/hide`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ state: checkbox.checked }),
              })
            ).text();
            if (checkbox.checked) {
              checkbox.parentElement?.parentElement?.classList.add(
                "gisst-Search-item-hidden",
              );
            } else {
              checkbox.parentElement?.parentElement?.classList.remove(
                "gisst-Search-item-hidden",
              );
            }
          } catch (error) {
            console.error(error);
            checkbox.checked = !checkbox.checked;
          }
        }
      })();
    });
    search.start();
  }
}

customElements.define("gisst-performance-search", GISSTPerformanceSearch);

interface WorkBib {
  work_name: string;
  work_platform: string;
  work_version: string;
  work_derived_from: string;
}

interface Work extends WorkBib {
  work_id: string;
}

interface InstanceWork extends Work {
  instance_id: string;
  environment_id: string;
}

interface Instance {
  instance_id: string;
  work_id: string;
  environment_id: string;
  instance_config: object | null;
  created_on: Date;
  derived_from_instance: string;
  derived_from_state: string;
}

interface Environment {
  environment_id: string;
  environment_name: string;
  environment_platform: string;
  environment_framework: string;
  environment_core_name: string;
  environment_core_version: string;
  environment_derived_from: string;
  environment_config: object | null;
  created_on: Date;
}

type Role = "content" | "dependency" | "config";

interface ObjectLink {
  object_id: string;
  object_role: Role;
  object_role_index: number;
  file_hash: string;
  file_filename: string;
  file_source_path: string;
  file_dest_path: string;
}

interface FullInstance {
  work: Work;
  info: Instance;
  environment: Environment;
  objects: ObjectLink[];
}

enum Upload {
  NotStarted,
  InProgress,
  Finished,
}

interface ExistingFile {
  existing: string;
}

interface FileUpload {
  file: File;
  upload: Upload;
  upload_result_id: string | null;
}

interface InstanceFile {
  ui_id: number;
  filename: string;
  source: ExistingFile | FileUpload;
  list_item: HTMLLIElement;
  input_element?: HTMLInputElement;
}

interface CoreInfo {
  core_name: string;
  core_version: string;
  core_platform: string;
}

function compute_hash(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const chunkSize = 16 * 1024 * 1024,
      chunks = Math.ceil(file.size / chunkSize),
      spark = new SparkMD5.ArrayBuffer(),
      fileReader = new FileReader();
    let currentChunk = 0;
    fileReader.onload = function (e) {
      spark.append(e.target!.result as ArrayBuffer);
      currentChunk++;

      if (currentChunk < chunks) {
        loadNext();
      } else {
        resolve(spark.end());
      }
    };
    fileReader.onerror = function () {
      reject(new Error("Hash computation error", { cause: fileReader.error }));
    };
    function loadNext() {
      const start = currentChunk * chunkSize,
        end = start + chunkSize >= file.size ? file.size : start + chunkSize;
      fileReader.readAsArrayBuffer(file.slice(start, end));
    }
    loadNext();
  });
}

class GISSTNewInstance extends HTMLElement {
  content_matcher: HTMLDivElement;
  base_url: string;
  next_ui_id: number;
  file_lists: {
    config: InstanceFile[];
    dependency: InstanceFile[];
    content: InstanceFile[];
  } = {
    config: [],
    dependency: [],
    content: [],
  };
  work: Work;
  instance: Instance;
  environment: Environment;
  env_form_v86_img_map: Record<V86ImageType, number | null>;
  env_form_mode: "form" | "manual";

  constructor() {
    super();

    this.work = {
      work_id: "",
      work_name: "",
      work_version: "",
      work_platform: "",
      work_derived_from: "",
    };
    this.instance = {
      instance_id: "",
      work_id: "",
      environment_id: "",
      instance_config: null,
      created_on: new Date(),
      derived_from_instance: "",
      derived_from_state: "",
    };
    this.environment = {
      environment_id: "",
      environment_platform: "",
      environment_name: "",
      environment_framework: "",
      environment_core_name: "",
      environment_core_version: "",
      environment_derived_from: "",
      environment_config: null,
      created_on: new Date(),
    };
    this.base_url = "/";
    this.next_ui_id = 0;
    this.content_matcher = document.createElement("div");
    this.env_form_v86_img_map = {
      hda: null,
      hdb: null,
      fda: null,
      fdb: null,
      cdrom: null,
    };
    this.env_form_mode = "form";
  }
  /*
    Workflow:
    1. Use an instance search to base on an existing work+env+instance OR give platform and try to match content file
    2. Fields:
    2.a. core selector (can upgrade core version here)
    2.b. work name
    2.c. work biblio...
    2.d. env config (for v86)
    2.e. instance files
    2.e.1. deps
    2.e.2. configs
    2.e.3. content
    3. If any section is modified or fresh add new record for that (server side choice)
   */
  connectedCallback() {
    const search_url = this.getAttribute("search-url");
    const search_key = this.getAttribute("search-key");
    const base_url = this.getAttribute("base-url");

    if (
      search_url == undefined ||
      search_key == undefined ||
      base_url == undefined
    ) {
      throw new Error(
        "Cannot create instance search UI without search url, search key, and base url",
      );
    }

    this.base_url = base_url;

    // Initialize templates
    this.content_matcher = elementFromTemplates("new-instance-method-two")
      .firstElementChild as HTMLDivElement;
    const search_method_div = elementFromTemplates("new-instance-method-one")
      .firstElementChild as HTMLDivElement;
    const metadata_form = elementFromTemplates("new-instance-details-form")
      .firstElementChild as HTMLFormElement;
    const select_files_form = elementFromTemplates(
      "new-instance-select-files-form",
    ).firstElementChild as HTMLFormElement;
    const submit_div = elementFromTemplates("new-instance-submit")
      .firstElementChild as HTMLDivElement;

    // config panels for form/manual entry tabs
    const config_form = elementFromTemplates("new-instance-config-form")
      .firstElementChild as HTMLDivElement;
    const missing_config_form_panel = elementFromTemplates(
      "missing-config-form-panel",
    ).firstElementChild as HTMLDivElement;
    const v86_config_form_panel = elementFromTemplates("v86-config-form-panel")
      .firstElementChild as HTMLDivElement;
    const ra_config_form_panel = elementFromTemplates("ra-config-form-panel")
      .firstElementChild as HTMLDivElement;
    const config_manual_panel = elementFromTemplates(
      "config-manual-entry-panel",
    ).firstElementChild as HTMLDivElement;

    // Setup method 1 search
    const search_box = search_method_div.querySelector(
      "#new-instance-search-box",
    ) as HTMLElement;
    const results_box = search_method_div.querySelector(
      "#new-instance-results-box",
    ) as HTMLElement;
    const results_header = search_method_div.querySelector(
      ".gisst-New-instance-search-header",
    ) as HTMLElement;
    const results_header_display = results_header.style.display;

    const search = search_instances(search_url, search_key, {});

    // Custom handler for hiding and showing table header
    const custom_stats = connectStats((renderOptions) => {
      if (renderOptions.nbHits <= 0 || !renderOptions.query) {
        results_header.style.display = "none";
      } else {
        results_header.style.display = results_header_display;
      }
    });

    search.addWidgets([
      widgets.searchBox({
        container: search_box,
      }),
      widgets.hits({
        container: results_box,
        transformItems(items, { results }) {
          // Don't show any results for empty query
          if (!results?.query) return [];
          return items;
        },
        templates: {
          item(hit, { html, components }) {
            return html`
              <button class="gisst-New-instance-search-item-button gisst-Search-results-row" onClick="${() => {
                console.log("selected ", hit);

                const work = hit as object as InstanceWork;
                self.update_work_bibinfo(work);
                void self.update_work_instanceenv_info(work.instance_id);

                // Set the search bar to the name of the work without showing any results
                search.helper!.setQuery("").search();
                search.helper!.setQuery(work.work_name);

                if (document.activeElement) {
                  (document.activeElement as HTMLElement).blur();
                }
              }}">
                <div
                  class="gisst-Search-cell gisst-New-instance-search-work-name"
                >
                  ${components.Highlight({ hit, attribute: "work_name" })}</span
                </div>
                <div
                  class="gisst-Search-cell gisst-New-instance-search-platform-info"
                >
                  ${components.Highlight({ hit, attribute: "work_platform" })}
                </div>
                <div
                  class="gisst-Search-cell gisst-New-instance-search-version-info"
                >
                  ${hit.work_version}
                </div>
              </button>
            `;
          },
          empty() {
            return null;
          },
        },
      }),
      widgets.configure({ hitsPerPage: 10 }),
      custom_stats({}),
    ]);

    search.start();

    // Setup method 2
    void this.init_core_chooser(
      this.content_matcher.getElementsByClassName(
        "core_chooser",
      )[0] as HTMLSelectElement,
    );

    /* eslint @typescript-eslint/no-this-alias: "off" */
    const self = this;

    const content_match_upload = this.content_matcher.getElementsByClassName(
      "content-match-upload",
    )[0] as HTMLInputElement;
    content_match_upload.disabled = true;
    let match_platform: string | null = null;
    const match_core_chooser: HTMLSelectElement =
      this.content_matcher.querySelector(".core_chooser")!;

    // Method 2 Event Listeners
    match_core_chooser.onchange = (event) => {
      content_match_upload.disabled = false;
      const first_option =
        match_core_chooser.firstElementChild! as HTMLOptionElement;
      if (first_option.value == "(init)") {
        match_core_chooser.removeChild(first_option);
      }
      match_platform = (event.target! as HTMLInputElement).value!;
    };

    const content_file_name =
      this.content_matcher.querySelector(".content-file-name");
    content_match_upload.onchange = (event) => {
      const files = (event.target! as HTMLInputElement).files ?? [];
      if (files.length == 1 && match_platform != null) {
        const file = files[0];
        self.clear_file_lists();
        compute_hash(file)
          .then((hash) => self.content_check_hash(match_platform!, file, hash))
          .catch((err) => {
            self.content_check_show_error(
              new Error("content hash check failed", { cause: err }),
            );
          });
        if (content_file_name != null) {
          content_file_name.innerHTML = file.name;
        }
      }
    };

    const core_chooser = metadata_form.getElementsByClassName(
      "core_chooser",
    )[0] as HTMLSelectElement;
    void this.init_core_chooser(core_chooser);
    core_chooser.onchange = () => {
      // update work platform, environment core
      const opt = core_chooser.selectedOptions[0];
      self.work.work_platform = opt.dataset["corePlatform"]!;
      self.environment.environment_platform = self.work.work_platform;
      self.environment.environment_core_name = opt.dataset["coreName"]!;
      self.environment.environment_core_version = opt.dataset["coreVersion"]!;
      self.environment.environment_framework =
        self.environment.environment_core_name == "v86" ? "v86" : "retroarch";
      this.env_form_reset();
    };

    metadata_form.querySelector<HTMLInputElement>(
      'input[name="work_name"]',
    )!.onchange = (evt) => {
      const target = evt.target as HTMLInputElement;
      self.work.work_name = target.value;
      self.environment.environment_name = target.value;
    };

    metadata_form.querySelector<HTMLInputElement>(
      'input[name="work_version"]',
    )!.onchange = (evt) => {
      const target = evt.target as HTMLInputElement;
      self.work.work_version = target.value;
    };

    v86_config_form_panel.querySelector<HTMLInputElement>(
      'textarea[name="env_config"]',
    )!.onchange = (evt) => {
      const target = evt.target as HTMLInputElement;
      self.environment.environment_config =
        target.value == "" || target.value == "null"
          ? {}
          : (JSON.parse(target.value) as object);
    };

    // other biblio stuff

    for (const input of select_files_form.querySelectorAll(
      ".instance-file-upload",
    ) as unknown as HTMLInputElement[]) {
      const lst = input.parentElement?.nextElementSibling;
      const group = input.getAttribute("data-target")!;
      input.onchange = (event) => {
        console.log(event, lst);
        const files = input.files ?? [];
        if (files.length == 0) {
          return;
        }
        self.add_to_file_list(
          group as Role,
          files[0].name,
          {
            to_upload: files[0],
          },
          input,
        );
      };
    }

    // V86 Environment Config Handling

    // initialize config method tabbing
    const tabsContainer = config_form.querySelector(".gisst-New-instance-tabs");
    const tabList = tabsContainer!.querySelector('[role="tablist"]');
    const tabs = Array.from(tabList!.querySelectorAll<HTMLButtonElement>('[role="tab"]'));
    const tabPanelsContainer = tabsContainer!.querySelector(".tab-panels");
    const tabPanels = Array.from(tabPanelsContainer!.querySelectorAll<HTMLDivElement>('[role="tabpanel"]'));

    const panel_1 = tabPanelsContainer!.querySelector(
      "#panel-1",
    ) as HTMLDivElement;
    const panel_2 = tabPanelsContainer!.querySelector(
      "#panel-2",
    ) as HTMLDivElement;

    panel_1.appendChild(missing_config_form_panel);
    panel_1.appendChild(v86_config_form_panel);
    panel_1.appendChild(ra_config_form_panel);
    panel_2.appendChild(config_manual_panel);

    // switch panel visibility
    tabs.forEach((tab) => {
      tab.addEventListener("click", (e) => {
        const btn = e.target as HTMLButtonElement;
        this.showTab(btn, tabs, tabPanels);

        if (btn.dataset.mode === "form") {
          this.env_form_mode = "form";
        } else {
          this.env_form_mode = "manual";
        }
        this.env_form_reset();
      });
    });


    (
      v86_config_form_panel.querySelector(
        ".gisst-New-instance-config-body .gisst-New-instance-config-form",
      ) as HTMLFormElement
    ).addEventListener("input", () => {
      this.env_form_generate();
    });

    // update file lists
    const disk_files_div = config_form.querySelectorAll(
      ".gisst-New-instance-disk-file-select",
    )!;
    disk_files_div.forEach((disk_select) => {
      const input = disk_select.querySelector("input") as HTMLInputElement;
      input.addEventListener("change", () => {
        const files = input.files ?? [];
        if (files.length == 0) return;

        self.add_to_file_list(
          "content",
          files[0].name,
          {
            to_upload: files[0],
          },
          input,
          this.env_form_v86_img_map[input.name as V86ImageType]!,
        );
      });
    });

    const button =
      submit_div.querySelector<HTMLButtonElement>("button.submit")!;
    const status =
      submit_div.querySelector<HTMLParagraphElement>("p.submit-status")!;

    button.onclick = async () => {
      let ready = true;

      // go through all files, make sure each one is uploaded or start the upload
      for (const role of Object.keys(self.file_lists)) {
        for (const [index, file] of self.file_lists[role as Role].entries()) {
          if (!("upload" in file.source)) {
            continue;
          }

          if (file.source.upload == Upload.NotStarted) {
            const upload_button =
              file.list_item.querySelector<HTMLButtonElement>("button.upload")!;
            const progress =
              file.list_item.querySelector<HTMLProgressElement>("progress")!;

            void this.upload_file(role as Role, index, upload_button, progress);
            // TODO: set a status message somewhere
            console.log("Start uploading", role, index, file);
            ready = false;
          } else if (file.source.upload == Upload.InProgress) {
            console.log("Still uploading", role, index, file);
            ready = false;
          }
        }
      }

      if (!ready) {
        status.textContent =
          'Please wait until all objects have been successfully uploaded, and then click on "Create Instance" again';
        return;
      }

      // try to create environment and work records (but no problem if they already exist)
      status.textContent = "Creating records...";
      try {
        const work_id = (
          (await (
            await fetch(`${this.base_url}/works/create`, {
              method: "POST",
              cache: "no-cache",
              headers: {
                "Content-Type": "application/json",
                Accept: "application/json",
              },
              body: JSON.stringify(this.work),
            })
          ).json()) as Work
        ).work_id;

        const environment_id = (
          (await (
            await fetch(`${this.base_url}/environments/create`, {
              method: "POST",
              cache: "no-cache",
              headers: {
                "Content-Type": "application/json",
                Accept: "application/json",
              },
              body: JSON.stringify(this.environment),
            })
          ).json()) as Environment
        ).environment_id;

        this.instance.work_id = work_id;
        this.instance.environment_id = environment_id;
        const configs = [];
        for (const conf of this.file_lists.config) {
          configs.push(
            "existing" in conf.source
              ? conf.source.existing
              : conf.source.upload_result_id,
          );
        }

        const dependencies = [];
        for (const dep of this.file_lists.dependency) {
          dependencies.push(
            "existing" in dep.source
              ? dep.source.existing
              : dep.source.upload_result_id,
          );
        }

        const content = [];
        for (const cont of this.file_lists.content) {
          content.push(
            "existing" in cont.source
              ? cont.source.existing
              : cont.source.upload_result_id,
          );
        }

        const instance_id = (
          (await (
            await fetch(`${this.base_url}/instances/create`, {
              method: "POST",
              cache: "no-cache",
              headers: {
                "Content-Type": "application/json",
                Accept: "application/json",
              },
              body: JSON.stringify({
                instance: this.instance,
                configs,
                dependencies,
                content,
              }),
            })
          ).json()) as Instance
        ).instance_id;

        // redirect to new instance id if successful
        window.location.href = `${self.base_url}/instances/${instance_id}`;
      } catch (error) {
        console.error(error);
        if (error instanceof Error) {
          status.textContent = error.toString();
        } else {
          status.textContent = JSON.stringify(error);
        }
      }
    };

    const contents = document.createElement("div");
    contents.className = "gisst-New-instance-container";
    contents.appendChild(elementFromTemplates("new-instance-header"));
    contents.appendChild(search_method_div);
    contents.appendChild(this.content_matcher);
    contents.appendChild(metadata_form);
    contents.appendChild(config_form);
    contents.appendChild(select_files_form);
    contents.appendChild(submit_div);
    this.appendChild(contents);

    this.env_form_reset();
  }

  async content_check_hash(platform: string, file: File, hash: string) {
    const filename = file.name;
    const match_result = this.content_matcher.getElementsByClassName(
      "content_match_result",
    )[0];
    match_result.textContent = `${filename}:${hash}`;
    const platform_esc = encodeURIComponent(platform);
    const filename_esc = encodeURIComponent(filename);
    const hash_esc = encodeURIComponent(hash);
    const core_chooser = document.querySelector<HTMLSelectElement>(
      "#work-info select[name=work_core_chooser]",
    )!;
    const match_core_chooser =
      this.content_matcher.querySelector<HTMLSelectElement>(".core_chooser")!;
    try {
      const resp = (await (
        await fetch(
          `${this.base_url}/lookup-work?platform=${platform_esc}&filename=${filename_esc}&md5=${hash_esc}`,
        )
      ).json()) as WorkBib;
      // resp has work bib fields
      // and if there is an existing instance we should use that
      if ("instance_id" in resp) {
        this.update_work_bibinfo(resp);
        await this.update_work_instanceenv_info(
          (resp as InstanceWork).instance_id,
        );
        match_result.textContent = "Matched existing instance";
      } else {
        core_chooser.selectedIndex = match_core_chooser.selectedIndex;
        this.update_work_bibinfo(resp);
        match_result.textContent = "Matched known work";
        this.add_to_file_list("content", file.name, { to_upload: file });
      }
    } catch (error) {
      console.log("No work found", error);
      core_chooser.selectedIndex = match_core_chooser.selectedIndex;
      this.update_work_bibinfo({
        work_name: filename,
        work_platform: platform,
        work_version: "",
        work_derived_from: "",
      });
      match_result.textContent = "No match found";
      this.add_to_file_list("content", file.name, { to_upload: file });
    }
  }
  content_check_show_error(err: Error) {
    console.error(err);
    this.content_matcher.getElementsByClassName(
      "content_match_result",
    )[0].textContent = `error computing hash`;
  }
  async init_core_chooser(elt: HTMLSelectElement) {
    const cores = (await (await fetch(`${this.base_url}/cores`)).json()) as {
      cores: CoreInfo[];
    };
    const default_option = document.createElement("option");
    default_option.disabled = true;
    default_option.selected = true;
    default_option.value = "";
    default_option.textContent = `-- Please select a platform --`;
    elt.appendChild(default_option);
    for (const core of cores.cores) {
      const option = document.createElement("option");
      option.value = core.core_platform;
      option.dataset["coreName"] = core.core_name;
      option.dataset["coreVersion"] = core.core_version;
      option.dataset["corePlatform"] = core.core_platform;
      option.textContent = `${core.core_platform} (${core.core_name})`;
      elt.appendChild(option);
    }
  }
  update_work_bibinfo(work: WorkBib) {
    const core_chooser = document.querySelector<HTMLSelectElement>(
      "#work-info select[name=work_core_chooser]",
    )!;
    const work_name = document.querySelector<HTMLInputElement>(
      "#work-info input[name=work_name]",
    )!;
    const work_version = document.querySelector<HTMLInputElement>(
      "#work-info input[name=work_version]",
    )!;
    // TODO: fetch other bibliographic info fields...
    if (core_chooser.selectedOptions.length == 0) {
      core_chooser.selectedIndex = Array.from(core_chooser.options).findIndex(
        (o) => o.dataset["platform"] == work.work_platform,
      );
    }
    work_name.value = work.work_name;
    work_version.value = work.work_version;
    this.work.work_platform = work.work_platform;
    this.work.work_name = work.work_name;
    this.work.work_version = work.work_version;
    const opt = core_chooser.selectedOptions[0];
    this.environment.environment_core_name = opt.dataset["coreName"]!;
    this.environment.environment_core_version = opt.dataset["coreVersion"]!;
    this.environment.environment_platform = this.work.work_platform;
    this.environment.environment_framework =
      this.environment.environment_core_name == "v86" ? "v86" : "retroarch";
    // TODO: set them here...
  }

  clear_file_lists() {
    this.file_lists.config.length = 0;
    this.file_lists.dependency.length = 0;
    this.file_lists.content.length = 0;
    this.env_form_v86_img_map = {
      hda: null,
      hdb: null,
      fda: null,
      fdb: null,
      cdrom: null,
    };

    document
      .querySelectorAll("#work-files .gisst-New-instance-select-file-lists")
      .forEach((lst) => {
        lst.innerHTML = "";
      });
  }

  async upload_file(
    role: Role,
    index: number,
    button: HTMLButtonElement,
    progress: HTMLProgressElement,
  ) {
    const file = this.file_lists[role][index];
    if ("existing" in file.source) {
      return;
    }
    const src = file.source;
    if (src.upload != Upload.NotStarted) {
      return;
    }
    src.upload = Upload.InProgress;
    button.disabled = true;
    progress.max = 100;
    try {
      const hash = await compute_hash(src.file);
      const file_id = await new Promise((resolve, reject) => {
        const upload = new tus.Upload(src.file, {
          endpoint: `${this.base_url}/resources`,
          retryDelays: [0, 3000, 5000, 10000],
          chunkSize: 10485760,
          metadata: {
            filename: file.filename,
            hash,
          },
          onError: reject,
          onProgress: (uploaded, total) => {
            progress.value = (uploaded / total) * 100;
          },
          onSuccess: () => {
            const url_parts = upload.url!.split("/");
            const uuid_string = url_parts[url_parts.length - 1];
            progress.value = 100;
            src.upload_result_id = uuid_string;
            resolve(uuid_string);
          },
        });
        upload.start();
      });
      const object_id = (
        (await (
          await fetch(`${this.base_url}/objects/create`, {
            method: "POST",
            cache: "no-cache",
            headers: {
              "Content-Type": "application/json",
              Accept: "application/json",
            },
            body: JSON.stringify({
              file_id,
              object_description: file.filename,
            }),
          })
        ).json()) as ObjectLink
      ).object_id;
      src.upload = Upload.Finished;
      src.upload_result_id = object_id;
    } catch (error) {
      console.error(error);
      button.disabled = false;
      button.textContent = "Retry";
      src.upload = Upload.NotStarted;
      src.upload_result_id = null;
      progress.value = 0;
    }
  }

  /**
   * Check whether a file list member should be allowed to move.
   * @param role The role of the file to check
   * @returns True if the movement buttons should show. False otherwise.
   */
  file_list_should_allow_move(role: Role) {
    if (role !== "content") return true;
    return (
      this.environment.environment_framework !== "v86" ||
      this.env_form_mode !== "form"
    );
  }

  /**
   * Updates whether the movement buttons should show on the
   * file list elements based on the current state of the form.
   */
  file_list_update_move() {
    for (const [role, file_list] of Object.entries(this.file_lists)) {
      for (const file of file_list) {
        const move_up = file.list_item.querySelector(
          ".gisst-New-instance-move-up",
        ) as HTMLElement;
        const move_down = file.list_item.querySelector(
          ".gisst-New-instance-move-down",
        ) as HTMLElement;
        move_up.style.visibility = this.file_list_should_allow_move(
          role as Role,
        )
          ? ""
          : "hidden";
        move_down.style.visibility = this.file_list_should_allow_move(
          role as Role,
        )
          ? ""
          : "hidden";
      }
    }
  }

  /**
   * Sync the state of the environment config form with the selected platform and form mode.
   */
  env_form_state_sync() {
    // Show correct form based on page state
    const missing_form_body = document.querySelector(
      "#missing-env-form-body",
    ) as HTMLElement;
    const v86_form_body = document.querySelector(
      "#v86-env-form-body",
    ) as HTMLElement;
    const ra_form_body = document.querySelector(
      "#ra-env-form-body",
    ) as HTMLElement;
    switch (this.environment.environment_framework) {
      case "v86":
        v86_form_body.style.display = "";
        missing_form_body.style.display = "none";
        ra_form_body.style.display = "none";
        break;

      case "retroarch":
        v86_form_body.style.display = "none";
        missing_form_body.style.display = "none";
        ra_form_body.style.display = "";
        break;

      default:
        v86_form_body.style.display = "none";
        missing_form_body.style.display = "";
        ra_form_body.style.display = "none";
        break;
    }

    // Disable content file upload if v86 form, since they should use the form to upload instead
    const content_file_upload = document.querySelector(
      "#instance-content-upload",
    ) as HTMLInputElement;
    content_file_upload.disabled =
      this.env_form_mode === "form" &&
      this.environment.environment_framework === "v86";

    this.file_list_update_move();
  }

  /**
   * Reset the env form.
   */
  env_form_reset() {
    if (
      this.environment.environment_framework === "v86" &&
      this.env_form_mode === "form"
    ) {
      this.clear_file_lists();
      this.environment.environment_config = {};
      this.env_form_existing_sync();
      this.env_form_generate();
    }

    this.env_form_state_sync();
  }

  add_to_file_list(
    role: Role,
    filename: string,
    source: { existing: string } | { to_upload: File },
    input_element?: HTMLInputElement,
    index?: number,
  ) {
    const is_existing = "existing" in source;
    const lst = document.querySelector<HTMLOListElement>(
      `#work-files p:has(input[data-target=${role}]) + ol`,
    )!;
    const line_item = document.createElement("li");
    const file_record = {
      ui_id: this.next_ui_id++,
      filename,
      source: is_existing
        ? source
        : {
            file: source.to_upload,
            upload: Upload.NotStarted,
            upload_progress: 0,
            upload_result_id: null,
          },
      list_item: line_item,
      input_element,
    };

    line_item.innerHTML = `
      <div class="gisst-New-instance-line-item-top">${filename} <button type="button" class="remove gisst-New-instance-line-item-buttons" title="Remove">X</button></div>
    `;

    const line_item_bottom_div = document.createElement("div");
    line_item_bottom_div.classList.add("gisst-New-instance-line-item-bottom");
    line_item.appendChild(line_item_bottom_div);

    const line_item_upload_div = document.createElement("div");
    line_item_bottom_div.appendChild(line_item_upload_div);

    const line_item_move_div = document.createElement("div");
    line_item_bottom_div.appendChild(line_item_move_div);

    if (is_existing) {
      const exist_note = document.createElement("span");
      exist_note.classList.add("exists-note");
      exist_note.textContent = "Existing file";
      line_item_upload_div.appendChild(exist_note);
    } else {
      // Upload button and progress bar
      const upload_status = document.createElement("button");
      upload_status.classList.add("upload");
      upload_status.classList.add("gisst-New-instance-line-item-buttons");
      upload_status.title = "Upload";
      upload_status.textContent = "U";
      upload_status.onclick = (_evt) => {
        const idx = this.file_lists[role].findIndex(
          (fr) => fr.ui_id == file_record.ui_id,
        );
        void this.upload_file(role, idx, upload_status, progress);
      };

      const progress = document.createElement("progress");
      progress.max = 0;
      progress.value = 0;

      line_item_upload_div.appendChild(upload_status);
      line_item_upload_div.appendChild(progress);
    }

    const move_up_btn = document.createElement("button");
    move_up_btn.type = "button";
    move_up_btn.classList.add("gisst-New-instance-move-up");
    move_up_btn.classList.add("gisst-New-instance-line-item-buttons");
    move_up_btn.textContent = "↑";
    move_up_btn.title = "Move up";
    move_up_btn.style.visibility = this.file_list_should_allow_move(role)
      ? ""
      : "hidden";
    line_item_move_div.appendChild(move_up_btn);

    const move_down_btn = document.createElement("button");
    move_down_btn.type = "button";
    move_down_btn.classList.add("gisst-New-instance-move-down");
    move_down_btn.classList.add("gisst-New-instance-line-item-buttons");
    move_down_btn.textContent = "↓";
    move_down_btn.title = "Move down";
    move_down_btn.style.visibility = this.file_list_should_allow_move(role)
      ? ""
      : "hidden";
    line_item_move_div.appendChild(move_down_btn);

    line_item.querySelector<HTMLButtonElement>(
      "button.gisst-New-instance-move-up",
    )!.onclick = (_evt) => {
      if (!this.file_list_should_allow_move(role)) return;

      const idx = this.file_lists[role].findIndex(
        (fr) => fr.ui_id == file_record.ui_id,
      );
      if (idx > 0) {
        const tmp = this.file_lists[role][idx - 1];
        this.file_lists[role][idx - 1] = file_record;
        this.file_lists[role][idx] = tmp;

        lst.removeChild(line_item);
        lst.insertBefore(line_item, lst.children[idx - 1]);
      }
    };

    line_item.querySelector<HTMLButtonElement>(
      "button.gisst-New-instance-move-down",
    )!.onclick = (_evt) => {
      if (!this.file_list_should_allow_move(role)) return;

      const idx = this.file_lists[role].findIndex(
        (fr) => fr.ui_id == file_record.ui_id,
      );
      if (idx < this.file_lists[role].length - 1) {
        const tmp = this.file_lists[role][idx + 1];
        this.file_lists[role][idx + 1] = file_record;
        this.file_lists[role][idx] = tmp;

        const swap_item = lst.children.item(idx + 1)!;
        lst.removeChild(swap_item);
        lst.insertBefore(swap_item, line_item);
      }
    };
    line_item.querySelector<HTMLButtonElement>("button.remove")!.onclick = (
      _evt,
    ) => {
      const idx = this.file_lists[role].findIndex(
        (fr) => fr.ui_id == file_record.ui_id,
      );
      const removed_instance_file = this.file_lists[role][idx];

      this.file_lists[role].splice(idx, 1);
      lst.removeChild(line_item);

      // Clear file selector if known
      const input_element = removed_instance_file.input_element;
      if (input_element) {
        input_element.value = "";
      }

      // Remove mapping in v86 form and regenerate config
      for (const [key, img_idx] of Object.entries(this.env_form_v86_img_map)) {
        const image_type = key as V86ImageType;
        if (img_idx === idx) {
          this.env_form_image_remove(image_type);
          break;
        }
      }
    };

    // Replace specified position or append to the end
    const desired_position = index ?? Infinity;
    const position = Math.min(desired_position, lst.children.length);

    const old_child = this.file_lists[role].at(position);
    if (old_child) {
      lst.insertBefore(line_item, old_child.list_item);
      lst.removeChild(old_child.list_item);
    } else {
      lst.appendChild(line_item);
    }
    this.file_lists[role].splice(position, 1, file_record);
  }

  /**
   * Prefill the add work form with information from an existing instance
   * @param instance_id The ID of the existing instance.
   */
  async update_work_instanceenv_info(instance_id: string) {
    const full_instance = (await (
      await fetch(
        `${this.base_url}/instances/${encodeURIComponent(instance_id)}`,
        { headers: [["Accept", "application/json"]] },
      )
    ).json()) as FullInstance;

    this.environment = full_instance.environment;
    const core_chooser = document.querySelector<HTMLSelectElement>(
      "#work-info select[name=work_core_chooser]",
    )!;
    core_chooser.selectedIndex = Array.from(core_chooser.options).findIndex(
      (o) =>
        o.dataset["coreName"] == this.environment.environment_core_name &&
        o.dataset["corePlatform"] == this.environment.environment_platform,
    );
    this.instance = full_instance.info;
    this.work = full_instance.work;

    const env_config_string = JSON.stringify(
      full_instance.environment.environment_config || {},
      null,
      2,
    );
    const env_configs = Array.from(
      document.querySelectorAll<HTMLTextAreaElement>("textarea[name=env_config]"),
    );
    for (const env_config of env_configs) {
      env_config.value = env_config_string;
    }

    this.clear_file_lists();
    full_instance.objects.sort(
      (a: ObjectLink, b: ObjectLink) =>
        a.object_role_index - b.object_role_index,
    );
    for (const lnk of full_instance.objects) {
      this.add_to_file_list(lnk.object_role, lnk.file_filename, {
        existing: lnk.object_id,
      });
    }

    // TODO: fetch other bibliographic info fields...

    this.env_form_existing_sync();
  }

  showTab(
    targetTab: HTMLButtonElement,
    tabs: HTMLButtonElement[],
    tabPanels: HTMLDivElement[],
  ) {
    for (const tab of tabs) {
      // select/unselect tabs
      if (tab === targetTab) continue;
      tab.setAttribute('aria-selected', 'false');
      tab.tabIndex = -1;
    }
    targetTab.setAttribute("aria-selected", 'true');
    targetTab.tabIndex = 0;

    // hide/show tab panels
    const targetTabPanel = document.getElementById(
      targetTab.getAttribute("aria-controls")!,
    );
    for (const panel of tabPanels) {
      if (panel === targetTabPanel) continue;
      panel.hidden = true;
    }
    targetTabPanel!.hidden = false;
  }


  /**
   * Parse an environment config JSON and populate the form based on its value.
   */
  env_form_existing_sync() {
    const env_config = this.environment.environment_config as V86Options;

    // Populate memory sizes
    const mem_size_input = document.querySelector(
      "#memory-size-select",
    ) as HTMLInputElement;
    mem_size_input.value =
      env_config?.memory_size !== undefined
        ? String(Math.round(env_config.memory_size / 1048576))
        : "";

    const gpu_mem_size_input = document.querySelector(
      "#gpu-memory-select",
    ) as HTMLInputElement;
    gpu_mem_size_input.value =
      env_config?.vga_memory_size !== undefined
        ? String(Math.round(env_config.vga_memory_size / 1048576))
        : "";

    // Populate BIOS
    const bios_chooser = document.querySelector(
      "#bios-chooser",
    ) as HTMLInputElement;
    bios_chooser.value = "bios-seabios";
    if (env_config?.bios?.url === "bochs-bios.bin") {
      bios_chooser.value = "bios-bochs";
    }

    const vga_bios_chooser = document.querySelector(
      "#vga-bios-chooser",
    ) as HTMLInputElement;
    vga_bios_chooser.value = "vga-bios-standard";
    if (env_config?.bios?.url === "bochs-vgabios.bin") {
      vga_bios_chooser.value = "vga-bios-bochs";
    }

    // Update file selects
    const images: Record<string, V86ImageType> = {
      "#hard-disk-a-upload ~ p.content-file-name": "hda",
      "#hard-disk-b-upload ~ p.content-file-name": "hdb",
      "#floppy-a-upload ~ p.content-file-name": "fda",
      "#floppy-b-upload ~ p.content-file-name": "fdb",
      "#cd-rom-upload ~ p.content-file-name": "cdrom",
    };
    for (const [query, image_type] of Object.entries(images)) {
      const element = document.querySelector(query)!;
      this.env_form_file_sync(element, env_config?.[image_type], image_type);
    }

    this.env_form_state_sync();
  }

  /**
   * Sync an existing file with the v86 environment form. The corresponding file name
   * label is updated, along with the map between content file indicies and images.
   * The V86Image should have a url matching `$CONTENT{X}` where X is an index into the
   * content files. `$CONTENT` is also recognised as `$CONTENT0`.
   * @param label The element to update the text of.
   * @param image The V86Image to match to a file.
   * @param image_type The type of the V86Image that we are syncing.
   */
  env_form_file_sync(
    label: Element,
    image?: V86Image,
    image_type?: V86ImageType,
  ) {
    label.textContent = "No file chosen";
    if (!image || !image_type) return;

    // Extract content number, i.e. "$CONTENT{X}"
    const parsed = image.url.match(/^\$CONTENT(\d*)$/m);
    const index = parsed?.[1];
    if (index === undefined) return;

    // "" is parsed into 0, so $CONTENT is ok
    const index_num = Number(index);
    const filename = this.file_lists.content.at(Number(index))?.filename;
    if (!filename) return;

    this.env_form_v86_img_map[image_type] = Number(index_num);
    label.textContent = filename;
  }

  /**
   * Generate a new environment config and update the form.
   */
  env_form_generate() {
    const form = document.querySelector(
      ".gisst-New-instance-config-body .gisst-New-instance-config-form",
    ) as HTMLFormElement;
    const form_data = new FormData(form);

    const hda = form_data.get("hda") as File;
    const hdb = form_data.get("hdb") as File;
    const fda = form_data.get("fda") as File;
    const fdb = form_data.get("fdb") as File;
    const cdrom = form_data.get("cdrom") as File;

    const memory_size_mib = Number(form_data.get("mem-size"));
    const vga_memory_size_mib = Number(form_data.get("gpu-memory"));

    const bios = form_data.get("bios") as "bios-seabios" | "bios-bochs";
    const vga_bios = form_data.get("vga-bios") as
      | "vga-bios-standard"
      | "vga-bios-bochs";

    // Build env config from form data
    let config: V86Options = {
      memory_size: memory_size_mib * 1048576,
      vga_memory_size: vga_memory_size_mib * 1048576,
      bios: {
        url: bios === "bios-seabios" ? "seabios.bin" : "bochs-bios.bin",
      },
      vga_bios: {
        url:
          vga_bios === "vga-bios-standard"
            ? "vgabios.bin"
            : "bochs-vgabios.bin",
      },
    };

    // Build $CONTENT0, $CONTENT1, etc mapping from present files in order
    const images: Record<V86ImageType, File> = {
      hda,
      hdb,
      fda,
      fdb,
      cdrom,
    };
    for (const [key, file] of Object.entries(images)) {
      const image_type = key as V86ImageType;
      const file_label = form.querySelector(`input[name="${image_type}"] ~ p`)!;
      // Note it is possible for the input element to have no file selected but
      // still be present in the map as an existing one.
      if (file.name === "" && this.env_form_v86_img_map[image_type] === null) {
        this.env_form_file_sync(file_label);
        continue;
      }

      // Image doesn't exist, so assign it the next content number
      if (this.env_form_v86_img_map[image_type] === null) {
        this.env_form_v86_img_map[image_type] = this.file_lists.content.length;
      }
      config[image_type] = {
        url: `$CONTENT${this.env_form_v86_img_map[image_type]}`,
        async: true,
      };

      if (file.name === "") {
        // Existing file, so file name needs to be synced from content file list
        this.env_form_file_sync(file_label, config[image_type], image_type);
      } else {
        file_label.textContent = file.name;
      }
    }

    const preview_element = document.querySelector(
      'textarea[name="env_config"]',
    ) as HTMLInputElement;
    preview_element.value = JSON.stringify(config, null, 2);
    this.environment.environment_config = config;
  }

  env_form_image_remove(image_type: V86ImageType) {
    const removed_idx = this.env_form_v86_img_map[image_type];
    if (removed_idx === null) return;
    this.env_form_v86_img_map[image_type] = null;

    // Subtract one from the remaining indicies that match the one removed
    for (const [key, idx] of Object.entries(this.env_form_v86_img_map)) {
      const image_type = key as V86ImageType;
      if (idx && idx > removed_idx) {
        --this.env_form_v86_img_map[image_type]!;
      }
    }

    this.env_form_generate();
  }
}

customElements.define("gisst-new-instance", GISSTNewInstance);
