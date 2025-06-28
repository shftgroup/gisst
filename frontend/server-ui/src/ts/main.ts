import { instantMeiliSearch, InstantMeiliSearchOptions } from '@meilisearch/instant-meilisearch';
import { default as instantsearch, InstantSearch } from 'instantsearch.js';
import 'instantsearch.css/themes/reset.css';
import '../css/server-ui-main.css';
import '../css/server-ui-search.css';
export type SearchOptions = InstantMeiliSearchOptions;
export * as search from 'instantsearch.js';
import * as widgets from 'instantsearch.js/es/widgets';
export * as widgets from 'instantsearch.js/es/widgets';

function search_instances(search_host:string, search_key:string, params:SearchOptions):InstantSearch {
  const { searchClient } = instantMeiliSearch(search_host, search_key, params);
  return instantsearch({
    indexName: 'instance',
    // @ts-expect-error instantsearch.js types are bad vis-a-vis instant-meilisearch
    searchClient
  });
}

export function search_states(search_host:string, search_key:string, params:SearchOptions):InstantSearch {
  const { searchClient } = instantMeiliSearch(search_host, search_key, params);
  return instantsearch({
    indexName: 'state',
    // @ts-expect-error instantsearch.js types are bad vis-a-vis instant-meilisearch
    searchClient
  });
}

export function search_saves(search_host:string, search_key:string, params:SearchOptions):InstantSearch {
  const { searchClient } = instantMeiliSearch(search_host, search_key, params);
  return instantsearch({
    indexName: 'save',
    // @ts-expect-error instantsearch.js types are bad vis-a-vis instant-meilisearch
    searchClient
  });
}

export function search_replays(search_host:string, search_key:string, params:SearchOptions):InstantSearch {
  const { searchClient } = instantMeiliSearch(search_host, search_key, params);
  return instantsearch({
    indexName: 'replay',
    // @ts-expect-error instantsearch.js types are bad vis-a-vis instant-meilisearch
    searchClient
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
    if(!search_url || !search_key || !base_url) {
      throw "Cannot create instance search UI without search url, search key, and base url";
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
        container: search_box
      }),
        widgets.hits({
          container: results_body,
          templates: {
            item(hit, { html, components }) {
              return html`
                <div class="gisst-Search-results-row">
                  <div class="gisst-Search-cell gisst-Search-instance-work-name">
                    <a href="${base_url}/instances/${hit.instance_id}">${components.Highlight({hit, attribute: "work_name"})}</a>
                  </div>
                  <div class="gisst-Search-cell gisst-Search-instance-platform-info">${components.Highlight({hit, attribute: "work_platform"})}</div>
                  <div class="gisst-Search-cell gisst-Search-instance-version-info">${hit.work_version}</div>
                  <div class="gisst-Search-cell gisst-Search-instance-actions-cell">
                    <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only" href="${base_url}/play/${hit.instance_id}">Play</a>
                    <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/play/${hit.instance_id}" title="Play">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polygon points="5 3 19 12 5 21 5 3"></polygon>
                      </svg>
                    </a>
                  </div>
                </div>
              `;
            },
          },
        }),
        widgets.configure({ hitsPerPage: 10 }),
        widgets.pagination({ container: pagination }),
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
    if(!search_url || !search_key || !base_url) {
      throw "Cannot create state search UI without search url, search key, and base url";
    }
    const limit_to_instance = this.getAttribute("instance-id");
    const limit_to_creator = this.getAttribute("creator-id");
    const show_creator_info = (this.getAttribute("creator-info") ?? "true") == "true";
    const show_instance_info = (this.getAttribute("instance-info") ?? "true") == "true";
    const can_clone = (this.getAttribute("can-clone") ?? "false") === 'true' ;
    const filters = [];
    if (limit_to_instance && limit_to_instance != "") {
      filters.push(`instance_id = "${limit_to_instance}"`);
    }
    if (limit_to_creator && limit_to_creator != "") {
      filters.push(`creator_id = "${limit_to_creator}"`);
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
          <div class="gisst-Search-header-cell gisst-Search-state-actions-cell">Actions</div>
      </div>
    `;
    if(!show_instance_info){
      results_container.querySelector(".gisst-Search-instance-info")!.remove();
      results_container.querySelector(".gisst-Search-results-header")!.classList.add("gisst-Search-no-instance");
    }
    if(!show_creator_info){
      results_container.querySelector(".gisst-Search-creator-info")!.remove();
      results_container.querySelector(".gisst-Search-results-header")!.classList.add("gisst-Search-no-creator");
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
        container: search_box
      }),
      widgets.configure({ hitsPerPage: 10, filters: filters.join(" AND ") }),
      widgets.pagination({ container: pagination }),
      /* TODO: these nested templates are gnarly, what can be done? */
      widgets.hits({
        container: results_body,
        templates: {
          item: (hit, { html, components }) => html`
           <div class="gisst-Search-results-row gisst-Search-responsive-results-row
           ${!show_creator_info ? "gisst-Search-no-creator":""} 
           ${!show_instance_info ? "gisst-Search-no-instance":""}">
              <div class="gisst-Search-cell gisst-Search-screenshot-cell">
                <img class="gisst-Search-screenshot" src="data:image/png;base64,${hit.screenshot_data}" alt="${hit.state_description} from instance ${hit.work_name}"/>
              </div>
              <div class="gisst-Search-cell gisst-Search-state-info">
                <div class="gisst-Search-name">
                  <a href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}">${components.Highlight({hit, attribute: "state_name"})}</a>
                </div>
                <div class="gisst-Search-description">
                  ${components.Highlight({hit, attribute: "state_description"})}
                </div>
              </div>
              ${show_instance_info ? html`
                <div class="gisst-Search-cell gisst-Search-instance-info">
                  <div class="gisst-Search-instance-name">
                    <a href="${base_url}/instances/${hit.instance_id}">${components.Highlight({hit, attribute: "work_name"})}</a>
                  </div>
                  <div class="gisst-Search-instance-details">
                    ${hit.work_version} • ${hit.work_platform}
                  </div>
                </div>
              ` : ""}
              ${show_creator_info ? html `
                <div class="gisst-Search-cell gisst-Search-creator-info">
                  <a href="${base_url}/creators/${hit.creator_id}">${components.Highlight({ hit, attribute: "creator_username"})}</a>
                </div>
              ` : ""}
              <div class="gisst-Search-cell gisst-Search-state-actions-cell">
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only" href="${base_url}/play/${hit.instance_id}">Play</a>
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/play/${hit.instance_id}" title="Play">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polygon points="5 3 19 12 5 21 5 3" />
                  </svg>
                </a>
                <a class="gisst-Search-btn gisst-Search-btn-secondary gisst-Search-btn-text-only" href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}&boot_into_record=true">Record</a>
                <a class="gisst-Search-btn gisst-Search-btn-secondary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}&boot_into_record=true">
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="10"/>
                    <circle cx="12" cy="12" r="4" fill="currentColor"/>
                  </svg>
                </a>
                ${can_clone ? html`
                  <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only" href="${hit.instance_id}/clone?state=${hit.state_id}">Clone</a>
                  <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${hit.instance_id}/clone?state=${hit.state_id}">
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
                    </svg>                  
                  </a>
                ` : ""}
              </div>
            </div> 
          `
        }
      })
    ]);
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
    if(!search_url || !search_key || !base_url) {
      throw "Cannot create save search UI without search url, search key, and base url";
    }
    const limit_to_instance = this.getAttribute("instance-id");
    const limit_to_creator = this.getAttribute("creator-id");
    const show_creator_info = (this.getAttribute("creator-info") ?? "true") == "true";
    const show_instance_info = (this.getAttribute("instance-info") ?? "true") == "true";
    const filters = [];
    if (limit_to_instance && limit_to_instance != "") {
      filters.push(`instance_id = "${limit_to_instance}"`);
    }
    if (limit_to_creator && limit_to_creator != "") {
      filters.push(`creator_id = "${limit_to_creator}"`);
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
          <div class="gisst-Search-header-cell gisst-Search-state-actions-cell">Actions</div>
      </div>
    `;
    if(!show_instance_info){
      results_container.querySelector(".gisst-Search-instance-info")!.remove();
      results_container.querySelector(".gisst-Search-results-header")!.classList.add("gisst-Search-no-instance");
    }
    if(!show_creator_info){
      results_container.querySelector(".gisst-Search-creator-info")!.remove();
      results_container.querySelector(".gisst-Search-results-header")!.classList.add("gisst-Search-no-creator");
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
        container: search_box
      }),
      widgets.configure({ hitsPerPage: 10, filters: filters.join(" AND ") }),
      widgets.pagination({ container: pagination }),
      /* TODO: these nested templates are gnarly, what can be done? */
      widgets.hits({
        container: results_body,
        templates: {
          item: (hit, { html, components }) => html`
             <div class="gisst-Search-results-row gisst-Search-responsive-results-row
           ${!show_creator_info ? "gisst-Search-no-creator":""} 
           ${!show_instance_info ? "gisst-Search-no-instance":""}">
              <div class="gisst-Search-cell gisst-Search-save-info">
                <div class="gisst-Search-name">
                  <a href="${base_url}/play/${hit.instance_id}?save=${hit.save_id}">${components.Highlight({hit, attribute: "save_short_desc"})}</a>
                </div>
                <div class="gisst-Search-description">
                  ${components.Highlight({hit, attribute: "save_description"})}
                </div>
              </div>
              ${show_instance_info ? html`
                <div class="gisst-Search-cell gisst-Search-instance-info">
                  <div class="gisst-Search-instance-name">
                    <a href="${base_url}/instances/${hit.instance_id}">${components.Highlight({hit, attribute: "work_name"})}</a>
                  </div>
                  <div class="gisst-Search-instance-details">
                    ${hit.work_version} • ${hit.work_platform}
                  </div>
                </div>
              ` : ""}
              ${show_creator_info ? html `
                <div class="gisst-Search-cell gisst-Search-creator-info">
                  <a href="${base_url}/creators/${hit.creator_id}">${components.Highlight({ hit, attribute: "creator_username"})}</a>
                </div>
              ` : ""}
              <div class="gisst-Search-cell gisst-Search-state-actions-cell">
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only" href="${base_url}/play/${hit.instance_id}">Play</a>
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/play/${hit.instance_id}" title="Play">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polygon points="5 3 19 12 5 21 5 3" />
                  </svg>
                </a>
                <a class="gisst-Search-btn gisst-Search-btn-secondary gisst-Search-btn-text-only" href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}&boot_into_record=true">Record</a>
                <a class="gisst-Search-btn gisst-Search-btn-secondary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}&boot_into_record=true">
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="10"/>
                    <circle cx="12" cy="12" r="4" fill="currentColor"/>
                  </svg>
                </a>
              </div>
            </div>  
          `
        }
      })
    ]);
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
    if(!search_url || !search_key || !base_url) {
      throw "Cannot create performance search UI without search url, search key, and base url";
    }
    const limit_to_instance = this.getAttribute("instance-id");
    const limit_to_creator = this.getAttribute("creator-id");
    const show_creator_info = (this.getAttribute("creator-info") ?? "true") == "true";
    const show_instance_info = (this.getAttribute("instance-info") ?? "true") == "true";
    const filters = [];
    if (limit_to_instance && limit_to_instance != "") {
      filters.push(`instance_id = "${limit_to_instance}"`);
    }
    if (limit_to_creator && limit_to_creator != "") {
      filters.push(`creator_id = "${limit_to_creator}"`);
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
          <div class="gisst-Search-header-cell gisst-Search-state-actions-cell">Actions</div>
      </div>
    `;
    if(!show_instance_info){
      results_container.querySelector(".gisst-Search-instance-info")!.remove();
      results_container.querySelector(".gisst-Search-results-header")!.classList.add("gisst-Search-no-instance");
    }
    if(!show_creator_info){
      results_container.querySelector(".gisst-Search-creator-info")!.remove();
      results_container.querySelector(".gisst-Search-results-header")!.classList.add("gisst-Search-no-creator");
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
        container: search_box
      }),
      widgets.configure({ hitsPerPage: 10, filters: filters.join(" AND ") }),
      widgets.pagination({ container: pagination }),
      /* TODO: these nested templates are gnarly, what can be done? */
      widgets.hits({
        container: results_body,
        templates: {
          item: (hit, { html, components }) => html`
            <div class="gisst-Search-results-row gisst-Search-responsive-results-row
           ${!show_creator_info ? "gisst-Search-no-creator":""} 
           ${!show_instance_info ? "gisst-Search-no-instance":""}">
              <div class="gisst-Search-cell gisst-Search-performance-info">
                <div class="gisst-Search-name">
                  <a href="${base_url}/play/${hit.instance_id}?replay=${hit.replay_id}">${components.Highlight({hit, attribute: "replay_name"})}</a>
                </div>
                <div class="gisst-Search-cell gisst-Search-description">
                  ${components.Highlight({hit, attribute: "replay_description"})}
                </div>
              </div>
              ${show_instance_info ? html`
                <div class="gisst-Search-cell gisst-Search-instance-info">
                  <div class="gisst-Search-instance-name">
                    <a href="${base_url}/instances/${hit.instance_id}">${components.Highlight({hit, attribute: "work_name"})}</a>
                  </div>
                  <div class="gisst-Search-instance-details">
                    ${hit.work_version} • ${hit.work_platform}
                  </div>
                </div>
              ` : ""}
              ${show_creator_info ? html `
                <div class="gisst-Search-cell gisst-Search-creator-info">
                  <a href="${base_url}/creators/${hit.creator_id}">${components.Highlight({ hit, attribute: "creator_username"})}</a>
                </div>
              ` : ""}
              <div class="gisst-Search-cell gisst-Search-state-actions-cell">
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only" href="${base_url}/play/${hit.instance_id}">Play</a>
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/play/${hit.instance_id}" title="Play">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polygon points="5 3 19 12 5 21 5 3" />
                  </svg>
                </a>
                <a class="gisst-Search-btn gisst-Search-btn-secondary gisst-Search-btn-text-only" href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}&boot_into_record=true">Record</a>
                <a class="gisst-Search-btn gisst-Search-btn-secondary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}&boot_into_record=true">
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="10"/>
                    <circle cx="12" cy="12" r="4" fill="currentColor"/>
                  </svg>
                </a>
              </div>
            </div>  
        `
        }
      })
    ]);
    search.start();
  }
}

customElements.define("gisst-performance-search", GISSTPerformanceSearch);
