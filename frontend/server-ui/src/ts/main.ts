import { instantMeiliSearch, InstantMeiliSearchOptions } from '@meilisearch/instant-meilisearch';
import { meilisearchAutocompleteClient, getMeilisearchResults } from '@meilisearch/autocomplete-client';
import { default as instantsearch, InstantSearch } from 'instantsearch.js';
import { autocomplete } from '@algolia/autocomplete-js';
import { default as SparkMD5 } from 'spark-md5';
import 'instantsearch.css/themes/reset.css';
import '../css/server-ui-main.css';
import '../css/server-ui-new-instance.css';
import '../css/server-ui-search.css';
import '../css/server-ui-instance.css';
export type SearchOptions = InstantMeiliSearchOptions;
export * as search from 'instantsearch.js';
import * as widgets from 'instantsearch.js/es/widgets';
export * as widgets from 'instantsearch.js/es/widgets';

function search_instances(search_host:string, search_key:string, params:SearchOptions):InstantSearch {
  params.primaryKey = "instance_id";
  const { searchClient } = instantMeiliSearch(search_host, search_key, params);
  return instantsearch({
    indexName: 'instance',
    searchClient
  });
}

export function search_states(search_host:string, search_key:string, params:SearchOptions):InstantSearch {
  params.primaryKey = "state_id";
  const { searchClient } = instantMeiliSearch(search_host, search_key, params);
  return instantsearch({
    indexName: 'state',
    searchClient
  });
}

export function search_saves(search_host:string, search_key:string, params:SearchOptions):InstantSearch {
  params.primaryKey = "save_id";
  const { searchClient } = instantMeiliSearch(search_host, search_key, params);
  return instantsearch({
    indexName: 'save',
    searchClient
  });
}

export function search_replays(search_host:string, search_key:string, params:SearchOptions):InstantSearch {
  params.primaryKey = "replay_id";
  const { searchClient } = instantMeiliSearch(search_host, search_key, params);
  return instantsearch({
    indexName: 'replay',
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
    if(search_url == undefined || search_key == undefined || base_url == undefined) {
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
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only" href="${base_url}/data/${hit.instance_id}">Cite</a>
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/data/${hit.instance_id}" title="Cite">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
 <g transform="translate(-25.751 -58.132)">
  <circle cx="30.974" cy="63.355" r="5.2227" stroke-width=".26458"/>
  <path d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176" stroke-width=".27134"/>
 </g>
 <g transform="translate(-14.007 -58.132)">
  <circle cx="30.974" cy="63.355" r="5.2227" stroke-width=".26458"/>
  <path d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176" stroke-width=".27134"/>
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
        widgets.pagination({ container: pagination, padding: 0, showFirst:false, showLast: false }),
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
    if(search_url == undefined || search_key == undefined || base_url == undefined) {
      throw "Cannot create state search UI without search url, search key, and base url";
    }
    const limit_to_instance = this.getAttribute("instance-id");
    const limit_to_creator = this.getAttribute("creator-id");
    const show_creator_info = (this.getAttribute("creator-info") ?? "true") == "true";
    const show_instance_info = (this.getAttribute("instance-info") ?? "true") == "true";
    // TODO: Now that environment framework is part of the info, we don't need this attribute and instead can make it hit-by-hit
    const can_clone = (this.getAttribute("can-clone") ?? "false") === 'true' ;
    const filters:string[] = [];
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
        widgets.pagination({ container: pagination, padding: 0, showFirst:false, showLast: false }),
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
                  <a href="${base_url}/creators/${hit.creator_id}">${components.Highlight({ hit, attribute: "creator_full_name"})}</a>
                </div>
              ` : ""}
              <div class="gisst-Search-cell gisst-Search-state-actions-cell">
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only" href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}">Play</a>
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}" title="Play">
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
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only" href="${base_url}/data/${hit.instance_id}?state=${hit.state_id}">Cite</a>
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/data/${hit.instance_id}?state=${hit.state_id}" title="Cite">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
 <g transform="translate(-25.751 -58.132)">
  <circle cx="30.974" cy="63.355" r="5.2227" stroke-width=".26458"/>
  <path d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176" stroke-width=".27134"/>
 </g>
 <g transform="translate(-14.007 -58.132)">
  <circle cx="30.974" cy="63.355" r="5.2227" stroke-width=".26458"/>
  <path d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176" stroke-width=".27134"/>
 </g>
                </svg>
                </a>
                ${can_clone ? html`
                  <a class="gisst-Search-btn gisst-Search-btn-accent gisst-Search-btn-text-only" href="${hit.instance_id}/clone?state=${hit.state_id}">Clone</a>
                  <a class="gisst-Search-btn gisst-Search-btn-accent gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${hit.instance_id}/clone?state=${hit.state_id}">
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
    if(search_url == undefined || search_key == undefined || base_url == undefined) {
      throw "Cannot create save search UI without search url, search key, and base url";
    }
    const limit_to_instance = this.getAttribute("instance-id");
    const limit_to_creator = this.getAttribute("creator-id");
    const show_creator_info = (this.getAttribute("creator-info") ?? "true") == "true";
    const show_instance_info = (this.getAttribute("instance-info") ?? "true") == "true";
    const filters:string[] = [];
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
        widgets.pagination({ container: pagination, padding: 0, showFirst:false, showLast: false }),
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
                  <a href="${base_url}/creators/${hit.creator_id}">${components.Highlight({ hit, attribute: "creator_full_name"})}</a>
                </div>
              ` : ""}
              <div class="gisst-Search-cell gisst-Search-state-actions-cell">
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only" href="${base_url}/play/${hit.instance_id}?save=${hit.save_id}">Play</a>
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/play/${hit.instance_id}?save=${hit.save_id}" title="Play">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polygon points="5 3 19 12 5 21 5 3" />
                  </svg>
                </a>
                <a class="gisst-Search-btn gisst-Search-btn-secondary gisst-Search-btn-text-only" href="${base_url}/play/${hit.instance_id}?save=${hit.save_id}&boot_into_record=true">Record</a>
                <a class="gisst-Search-btn gisst-Search-btn-secondary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/play/${hit.instance_id}?save=${hit.save_id}&boot_into_record=true">
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="10"/>
                    <circle cx="12" cy="12" r="4" fill="currentColor"/>
                  </svg>
                </a>

                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only" href="${base_url}/data/${hit.instance_id}?save=${hit.save_id}">Cite</a>
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/data/${hit.instance_id}?save=${hit.save_id}" title="Cite">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
 <g transform="translate(-25.751 -58.132)">
  <circle cx="30.974" cy="63.355" r="5.2227" stroke-width=".26458"/>
  <path d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176" stroke-width=".27134"/>
 </g>
 <g transform="translate(-14.007 -58.132)">
  <circle cx="30.974" cy="63.355" r="5.2227" stroke-width=".26458"/>
  <path d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176" stroke-width=".27134"/>
 </g>
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
    if(search_url == undefined || search_key == undefined || base_url == undefined) {
      throw "Cannot create performance search UI without search url, search key, and base url";
    }
    const limit_to_instance = this.getAttribute("instance-id");
    const limit_to_creator = this.getAttribute("creator-id");
    const show_creator_info = (this.getAttribute("creator-info") ?? "true") == "true";
    const show_instance_info = (this.getAttribute("instance-info") ?? "true") == "true";
    const filters:string[] = [];
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
        widgets.pagination({ container: pagination, padding: 0, showFirst:false, showLast: false }),
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
                  <a href="${base_url}/creators/${hit.creator_id}">${components.Highlight({ hit, attribute: "creator_full_name"})}</a>
                </div>
              ` : ""}
              <div class="gisst-Search-cell gisst-Search-state-actions-cell">
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only" href="${base_url}/play/${hit.instance_id}?replay=${hit.replay_id}">Play</a>
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/play/${hit.instance_id}?replay=${hit.replay_id}" title="Play">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polygon points="5 3 19 12 5 21 5 3" />
                  </svg>
                </a>
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only" href="${base_url}/data/${hit.instance_id}?replay=${hit.replay_id}">Cite</a>
                <a class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only" href="${base_url}/data/${hit.instance_id}?replay=${hit.replay_id}" title="Cite">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
 <g transform="translate(-25.751 -58.132)">
  <circle cx="30.974" cy="63.355" r="5.2227" stroke-width=".26458"/>
  <path d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176" stroke-width=".27134"/>
 </g>
 <g transform="translate(-14.007 -58.132)">
  <circle cx="30.974" cy="63.355" r="5.2227" stroke-width=".26458"/>
  <path d="m32.322 58.603s1.6069 0.77464 3.8938 5.3045c0.38114 5.5264-3.5746 10.109-7.339 12.176" stroke-width=".27134"/>
 </g>
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

interface Work {
  work_id: string,
  work_name: string,
  work_platform: string,
  work_version: string,
}

interface InstanceWork extends Work {
  instance_id: string,
  environment_id: string
}

interface Instance {
  instance_id: string,
  work_id: string,
  environment_id: string,
  instance_config: any,
  created_on: Date,
  derived_from_instance: string | null,
  derived_from_state: string | null
}

interface Environment {
  enviroment_id: string,
  environment_name: string,
  environment_framework: string,
  environment_core_name: string,
  environment_core_version: string,
  environment_derived_from: string | null,
  environment_config: any,
  created_on: Date
}

type Role = "content" | "dependency" | "config";

interface ObjectLink {
  object_id: string,
  object_role: Role,
  object_role_index: number,
  file_hash: string,
  file_filename: string,
  file_source_path: string,
  file_dest_path: string
}

interface FullInstance {
  work:Work,
  info:Instance,
  environment:Environment,
  objects:ObjectLink[]
}

enum Upload {
  NotStarted,
  InProgress,
  Finished
}

interface InstanceFile {
  filename: string,
  source: {existing:string} | {file:File, upload:Upload, upload_progress:number, upload_result_id:string|null}
}

class GISSTNewInstance extends HTMLElement {
  content_matcher: HTMLDivElement;
  base_url: string;
  file_lists:{config:InstanceFile[], dependency:InstanceFile[], content:InstanceFile[]} = {
    "config":[],
    "dependency":[],
    "content":[]
  };
  constructor() {
    super();
    this.base_url="/";
    this.content_matcher = document.createElement("div");
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
    if(search_url == undefined || search_key == undefined || base_url == undefined) {
      throw "Cannot create instance search UI without search url, search key, and base url";
    }
    this.base_url = base_url;
    const search_container = document.createElement("div");
    search_container.setAttribute("class", "gisst-Search-container");
    const search_explanation = document.createElement("p");
    search_explanation.textContent = "Method 2: Search for an existing instance (you can change the work details or content files later).";
    search_container.appendChild(search_explanation);
    const search_box = document.createElement("div");
    search_container.appendChild(search_box);
    const results_box = document.createElement("div");
    search_container.appendChild(results_box);
    const search_client = meilisearchAutocompleteClient({
      url: search_url,
      apiKey: search_key
    });
    autocomplete({
      container: search_box,
      panelContainer: results_box,
      detachedMediaQuery: "none",
      placeholder: 'Search',
      // @ts-expect-error autocomplete-js types are bad vis-a-vis meilisearch-autocomplete
      getSources({ query }) {
        return [
          {
            sourceId: 'instance',
              getItemInputValue: ({ item }) => item.work_name as string,
              getItems() {
              return getMeilisearchResults({
                searchClient:search_client,
                queries: [
                  {
                    indexName: 'instance',
                    query,
                  },
                ],
              })
            },
            templates: {
              item({ item, html }) {
                return html`
<div class="gisst-new-instance-selector">
  <div>${item.work_name}</div>
  <div>${item.work_version}</div>
  <div>${item.work_platform}</div>
</div>`
              },
            },
            onSelect({item, setStatus, refresh, setIsOpen, ...others}) {
              console.log("selected ",item,others);
              self.update_work_bibinfo(item as any as Work);
              self.update_work_instanceenv_info(item as any as InstanceWork)
              setStatus('idle');
              setIsOpen(false);
              refresh();
              if (document.activeElement) {
                (document.activeElement as HTMLElement)!.blur();
              }
            }
          },
        ]
      },
    });
    this.content_matcher.innerHTML = `
<p>You can find bibliographic data by attempting to match your content file against a community-developed database (Method 1), or you can search for a similar existing work already in the GISST system (Method 2). Using either method will populate the fields below, or you can skip both methods and create a new work by hand.</p>
<label for="match_core_chooser">Method 1: What platform is this work for?</label>
<select name="match_core_chooser" class="core_chooser">
</select>
<label for="content_match_upload">Method 1: Search using the given main content file:</label>
<input type="file" class="content-match-upload"></input>
<p class="content_match_result"></p>
`;
    this.init_core_chooser(this.content_matcher.getElementsByClassName("core_chooser")[0]! as HTMLSelectElement);
    const self = this;
    const content_match_upload = this.content_matcher.getElementsByClassName("content-match-upload")[0]! as HTMLInputElement;
    content_match_upload.disabled = true;
    let match_platform:string|null = null;
    (this.content_matcher.querySelector(".core_chooser")! as HTMLSelectElement).onchange = (event) => {
      content_match_upload.disabled = false;
      match_platform = (event.target! as HTMLInputElement).value!;
    };
    content_match_upload.onchange = (event) => {
      const files = (event.target! as HTMLInputElement).files ?? [];
      if (files.length == 1 && match_platform != null) {
        const file = files[0];
        const filename = file.name;
        const chunkSize = 16*1024*1024,
        chunks = Math.ceil(file.size / chunkSize),
        spark = new SparkMD5.ArrayBuffer(),
        fileReader = new FileReader();
        let currentChunk = 0;
        fileReader.onload = function (e) {
          spark.append((e.target! as FileReader).result as ArrayBuffer);
          currentChunk++;

          if (currentChunk < chunks) {
            loadNext();
          } else {
            self.content_check_hash(match_platform!, filename, spark.end());
          }
        };

        fileReader.onerror = function () {
          self.content_check_show_error();
        };

        function loadNext() {
          var start = currentChunk * chunkSize,
          end = ((start + chunkSize) >= file.size) ? file.size : start + chunkSize;
          fileReader.readAsArrayBuffer(file.slice(start,end));
        }

        loadNext();
      }
    };
    const metadata_form = document.createElement("form");
    metadata_form.id = "work_info";
    metadata_form.innerHTML = `
<label for="work_core_chooser">Work platform:</label>
<select name="work_core_chooser" class="core_chooser">
</select>
<label for="work_name">Work name:</label>
<input name="work_name"></input>
<label for="work_version">Version:</label>
<input name="work_version"></input>
<label for="env_config" id="env_config">Environment config:</label>
<textarea name="env_config"></textarea>
<label for="instance_dep_upload">Instance dependencies:</label>
<input type="file" name="instance_dep_upload" class="instance-file-upload" data-target="dependency"></input>
<ol></ol>
<label for="instance_config_upload">Instance config files:</label>
<input type="file" name="instance_config_upload" class="instance-file-upload" data-target="config"></input>
<ol></ol>
<label for="instance_content_upload">Instance content files:</label>
<input type="file" name="instance_content_upload" class="instance-file-upload" data-target="content"></input>
<ol></ol>
<button class="submit" type="button">Create Instance</button>
`;
    this.init_core_chooser(metadata_form.getElementsByClassName("core_chooser")[0]! as HTMLSelectElement);
    for (const input of metadata_form.getElementsByClassName("instance-file-upload") as any) {
      const inp = input as HTMLInputElement;
      const lst = input.nextElementSibling;
      const group = input.getAttribute("data-target");
      inp.onchange = (event) => {
        console.log(event,lst);
        const files = inp.files ?? [];
        if (files.length == 0) { return; }
        self.add_to_file_list(group, files[0].name, {to_upload:files[0]});
      };
    }
    const button = metadata_form.querySelector("button.submit")! as HTMLButtonElement;
    button.onclick = () => {
      // upload objects using tus, show some spinners, replacing each file list entry with the uploaded object
      // create environment, work
      // create instance once objects are up
    };
    const contents = document.createElement("div");
    contents.appendChild(this.content_matcher);
    contents.appendChild(search_container);
    contents.appendChild(metadata_form);
    this.appendChild(contents);
  }
  async content_check_hash(platform:string, filename:string, hash:string) {
    this.content_matcher.getElementsByClassName("content_match_result")[0]!.textContent = `${filename}:${hash}`;
    const platform_esc = encodeURIComponent(platform);
    const filename_esc = encodeURIComponent(filename);
    const hash_esc = encodeURIComponent(hash);
    const resp = await (await fetch(`${this.base_url}/lookup-work?platform=${platform_esc}&filename=${filename_esc}&hash=${hash_esc}`)).json();
    // resp is a work, not necessarily an instancework
    this.update_work_bibinfo(resp);
  }
  content_check_show_error() {
    this.content_matcher.getElementsByClassName("content_match_result")[0]!.textContent = `error computing hash`;
  }
  async init_core_chooser(elt:HTMLSelectElement) {
    const cores = await (await fetch(`${this.base_url}/cores`)).json();
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
  async update_work_bibinfo(work:Work) {
    const core_chooser = document.querySelector("#work_info select[name=work_core_chooser]")! as HTMLSelectElement;
    const work_name = document.querySelector("#work_info input[name=work_name]")! as HTMLInputElement;
    const work_version = document.querySelector("#work_info input[name=work_version]")! as HTMLInputElement;
    // TODO: fetch other bibliographic info fields...
    core_chooser.value = work.work_platform;
    work_name.value = work.work_name;
    work_version.value = work.work_version;
    // TODO: set them here...
  }
  clear_file_lists() {
    this.file_lists.config.length = 0;
    this.file_lists.dependency.length = 0;
    this.file_lists.content.length = 0;
    document.querySelectorAll("#work_info .instance-file-upload + ol").forEach((lst) => {lst.innerHTML = '';});
  }
  add_to_file_list(role:Role, filename:string, source:{existing:string} | {to_upload:File}) {
    const is_existing = 'existing' in source;
    const lst = document.querySelector(`#work_info input[data-target=${role}] + ol`)! as HTMLOListElement;
    const file_record = {filename,source:is_existing ? source : {file:source.to_upload, upload:Upload.NotStarted, upload_progress:0, upload_result_id:null}};
    const line_item = document.createElement("li");
    line_item.innerHTML = `
${filename} <button type="button" class="move-up">^</button> <button type="button" class="move-down">v</button> <button type="button" class="remove">X</button>
`;
    (line_item.querySelector("button.move-up")! as HTMLButtonElement).onclick = (_evt) => {
      const idx = this.file_lists[role].indexOf(file_record);
      if (idx > 0) {
        const tmp = this.file_lists[role][idx-1];
        this.file_lists[role][idx-1] = file_record;
        this.file_lists[role][idx] = tmp;
        lst.removeChild(line_item);
        lst.insertBefore(line_item, lst.children[idx-1]);
      }
    };
    (line_item.querySelector("button.move-down")! as HTMLButtonElement).onclick = (_evt) => {
      const idx = this.file_lists[role].indexOf(file_record);
      if (idx < this.file_lists[role].length-1) {
        const tmp = this.file_lists[role][idx+1];
        this.file_lists[role][idx+1] = file_record;
        this.file_lists[role][idx] = tmp;
        const swap_item = lst.children.item(idx+1)!;
        lst.removeChild(swap_item);
        lst.insertBefore(swap_item, line_item);
      }
    };
    (line_item.querySelector("button.remove")! as HTMLButtonElement).onclick = (_evt) => {
      const idx = this.file_lists[role].indexOf(file_record);
      this.file_lists[role].splice(idx,1);
      lst.removeChild(line_item);
    };
    const existing_note = document.createElement("span");
    existing_note.textContent = is_existing ? "E" : "N";
    line_item.appendChild(existing_note);
    lst.appendChild(line_item);
    this.file_lists[role].push(file_record);
  }
  async update_work_instanceenv_info(work:InstanceWork) {
    const env_config = document.querySelector("#work_info textarea[name=env_config]")! as HTMLInputElement;
    const full_instance:FullInstance = await (await fetch(`${this.base_url}/instances/${encodeURIComponent(work.instance_id)}`)).json();
    env_config.value = JSON.stringify(full_instance.environment.environment_config);
    this.clear_file_lists();
    full_instance.objects.sort((a:ObjectLink,b:ObjectLink) => (a.object_role_index - b.object_role_index));
    for (const lnk of full_instance.objects) {
      this.add_to_file_list(lnk.object_role, lnk.file_filename, {existing:lnk.object_id});
    }
    // TODO make sure there is a hidden instance derived from field and update that
  }
}


customElements.define("gisst-new-instance", GISSTNewInstance);
