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
import * as tus from 'tus-js-client';

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
                  <div class="gisst-Search-cell gisst-Search-actions-cell">
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
    const active_creator_id = this.getAttribute("user-creator-id") ?? "";
    const active_creator_role = parseInt(this.getAttribute("user-role") ?? "100");
    const show_hidden_state = active_creator_role <= 10 || (limit_to_creator == active_creator_id && active_creator_id != "");
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
    if (!show_hidden_state) {
      filters.push('hidden = false');
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
          ${show_hidden_state ? '<div class="gisst-Search-header-cell gisst-Search-hidestate">Hidden</div>' : ''}
          <div class="gisst-Search-header-cell gisst-Search-actions-cell">Actions</div>
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
           ${hit.hidden ? "gisst-Search-item-hidden" : ""}
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
              ${show_hidden_state ? html`
                <div class="gisst-Search-cell gisst-Search-hidden-info">
                <input name="hidden-${hit.state_id}" class="hide-show-checkbox" type="checkbox" data-id="${hit.state_id}" checked="${hit.hidden}"></input>
                </div>
                ` : ""}
              <div class="gisst-Search-cell gisst-Search-actions-cell">
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

    results_body.addEventListener('click', async function(event) {
      if (event.target && (event.target! as HTMLElement).classList.contains('hide-show-checkbox')) {
        const checkbox = event.target! as HTMLInputElement;
        const id = checkbox.dataset["id"];
        try {
          await (await fetch(`${base_url}/states/${id}/hide`, {method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({"state":checkbox.checked})})).text();
          if (checkbox.checked) {
            checkbox.parentElement?.parentElement?.classList.add("gisst-Search-item-hidden");
          } else {
            checkbox.parentElement?.parentElement?.classList.remove("gisst-Search-item-hidden");
          }
        } catch(error) {
          console.error(error);
          checkbox.checked = !checkbox.checked;
        }
      }
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
    if(search_url == undefined || search_key == undefined || base_url == undefined) {
      throw "Cannot create save search UI without search url, search key, and base url";
    }
    const limit_to_instance = this.getAttribute("instance-id");
    const limit_to_creator = this.getAttribute("creator-id");
    const active_creator_id = this.getAttribute("user-creator-id") ?? "";
    const active_creator_role = parseInt(this.getAttribute("user-role") ?? "0");
    const show_creator_info = (this.getAttribute("creator-info") ?? "true") == "true";
    const show_hidden_state = active_creator_role <= 10 || (limit_to_creator == active_creator_id && active_creator_id != "");
    const show_instance_info = (this.getAttribute("instance-info") ?? "true") == "true";
    const filters:string[] = [];
    if (limit_to_instance && limit_to_instance != "") {
      filters.push(`instance_id = "${limit_to_instance}"`);
    }
    if (limit_to_creator && limit_to_creator != "") {
      filters.push(`creator_id = "${limit_to_creator}"`);
    }
    if (!show_hidden_state) {
      filters.push('hidden = false');
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
          ${show_hidden_state ? '<div class="gisst-Search-header-cell gisst-Search-hidestate">Hidden</div>' : ''}
          <div class="gisst-Search-header-cell gisst-Search-actions-cell">Actions</div>
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
           ${hit.hidden ? "gisst-Search-item-hidden" : ""}
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
              ${show_hidden_state ? html`
                <div class="gisst-Search-cell gisst-Search-hidden-info">
                <input name="hidden-${hit.state_id}" class="hide-show-checkbox" type="checkbox" data-id="${hit.state_id}" checked="${hit.hidden}"></input>
                </div>
                ` : ""}
              <div class="gisst-Search-cell gisst-Search-actions-cell">
                ${show_hidden_state ? html`
                <label for="hidden-${hit.save_id}" class="gisst-Search-checkbox-label">Hide?</label>
                <input name="hidden-${hit.save_id}" class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only hide-show-checkbox" type="checkbox" data-id="${hit.save_id}" checked="${hit.hidden}"></input>
                ` : ""}
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
    results_body.addEventListener('click', async function(event) {
      if (event.target && (event.target! as HTMLElement).classList.contains('hide-show-checkbox')) {
        const checkbox = event.target! as HTMLInputElement;
        const id = checkbox.dataset["id"];
        try {
          await (await fetch(`${base_url}/saves/${id}/hide`, {method:"POST", headers:{"Content-Type":"application/json"}, body:JSON.stringify({"state":checkbox.checked})})).text();
          if (checkbox.checked) {
            checkbox.parentElement?.parentElement?.classList.add("gisst-Search-item-hidden");
          } else {
            checkbox.parentElement?.parentElement?.classList.remove("gisst-Search-item-hidden");
          }
        } catch(error) {
          console.error(error);
          checkbox.checked = !checkbox.checked;
        }
      }
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
    if(search_url == undefined || search_key == undefined || base_url == undefined) {
      throw "Cannot create performance search UI without search url, search key, and base url";
    }
    const limit_to_instance = this.getAttribute("instance-id");
    const limit_to_creator = this.getAttribute("creator-id");
    const active_creator_id = this.getAttribute("user-creator-id") ?? "";
    const active_creator_role = parseInt(this.getAttribute("user-role") ?? "0");
    const show_creator_info = (this.getAttribute("creator-info") ?? "true") == "true";
    const show_hidden_state = active_creator_role <= 10 || (limit_to_creator == active_creator_id && active_creator_id != "");
    const show_instance_info = (this.getAttribute("instance-info") ?? "true") == "true";
    const filters:string[] = [];
    if (limit_to_instance && limit_to_instance != "") {
      filters.push(`instance_id = "${limit_to_instance}"`);
    }
    if (limit_to_creator && limit_to_creator != "") {
      filters.push(`creator_id = "${limit_to_creator}"`);
    }
    if (!show_hidden_state) {
      filters.push('hidden = false');
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
          ${show_hidden_state ? '<div class="gisst-Search-header-cell gisst-Search-hidestate">Hidden</div>' : ''}
          <div class="gisst-Search-header-cell gisst-Search-actions-cell">Actions</div>
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
           ${hit.hidden ? "gisst-Search-item-hidden" : ""}
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
              ${show_hidden_state ? html`
                <div class="gisst-Search-cell gisst-Search-hidden-info">
                <input name="hidden-${hit.state_id}" class="hide-show-checkbox" type="checkbox" data-id="${hit.state_id}" checked="${hit.hidden}"></input>
                </div>
                ` : ""}
              <div class="gisst-Search-cell gisst-Search-actions-cell">
                ${show_hidden_state ? html`
                <label for="hidden-${hit.replay_id}" class="gisst-Search-checkbox-label">Hide?</label>
                <input name="hidden-${hit.replay_id}" class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only hide-show-checkbox" type="checkbox" data-id="${hit.replay_id}" checked="${hit.hidden}"></input>
                ` : ""}
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

    results_body.addEventListener('click', async function(event) {
      if (event.target && (event.target! as HTMLElement).classList.contains('hide-show-checkbox')) {
        const checkbox = event.target! as HTMLInputElement;
        const id = checkbox.dataset["id"];
        try {
          await (await fetch(`${base_url}/replays/${id}/hide`, {method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({"state":checkbox.checked})})).text();
          if (checkbox.checked) {
            checkbox.parentElement?.parentElement?.classList.add("gisst-Search-item-hidden");
          } else {
            checkbox.parentElement?.parentElement?.classList.remove("gisst-Search-item-hidden");
          }
        } catch(error) {
          console.error(error);
          checkbox.checked = !checkbox.checked;
        }
      }
    });
    search.start();
  }
}

customElements.define("gisst-performance-search", GISSTPerformanceSearch);

interface WorkBib {
  work_name: string,
  work_platform: string,
  work_version: string,
  work_derived_from: string,
}

interface Work extends WorkBib {
  work_id: string,
}

interface InstanceWork extends Work {
  instance_id: string,
  environment_id: string
}

interface Instance {
  instance_id: string,
  work_id: string,
  environment_id: string,
  instance_config: object | null,
  created_on: Date,
  derived_from_instance: string,
  derived_from_state: string
}

interface Environment {
  environment_id: string,
  environment_name: string,
  environment_platform: string,
  environment_framework: string,
  environment_core_name: string,
  environment_core_version: string,
  environment_derived_from: string,
  environment_config: object | null,
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

interface ExistingFile {
  existing: string
}

interface FileUpload {
  file:File, upload:Upload, upload_result_id:string|null
}

interface InstanceFile {
  ui_id: number,
  filename: string,
  list_item: HTMLLIElement,
  source: ExistingFile | FileUpload
}

function compute_hash(file:File): Promise<string> {
  return new Promise((resolve, reject) => {
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
        resolve(spark.end());
      }
    };
    fileReader.onerror = function () {
      reject();
    };
    function loadNext() {
      const start = currentChunk * chunkSize,
        end = ((start + chunkSize) >= file.size) ? file.size : start + chunkSize;
      fileReader.readAsArrayBuffer(file.slice(start,end));
    }
    loadNext();
  });
}

class GISSTNewInstance extends HTMLElement {
  content_matcher: HTMLDivElement;
  base_url: string;
  next_ui_id: number;
  file_lists:{config:InstanceFile[], dependency:InstanceFile[], content:InstanceFile[]} = {
    "config":[],
    "dependency":[],
    "content":[]
  };
  work: Work;
  instance: Instance;
  environment: Environment;
  constructor() {
    super();
    this.work = {
      work_id: "",
      work_name: "",
      work_version: "",
      work_platform: "",
      work_derived_from: ""
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
      created_on: new Date()
    };
    this.base_url="/";
    this.next_ui_id = 0;
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
              self.update_work_bibinfo(item as object as InstanceWork);
              self.update_work_instanceenv_info((item as object as InstanceWork).instance_id)
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
  <option value="(init)">Select core...</option>
</select>
<label for="content_match_upload">Method 1: Search using the given main content file:</label>
<input type="file" class="content-match-upload"></input>
<p class="content_match_result"></p>
`;
    this.init_core_chooser(this.content_matcher.getElementsByClassName("core_chooser")[0]! as HTMLSelectElement);
    /* eslint @typescript-eslint/no-this-alias: "off" */
    const self = this;
    const content_match_upload = this.content_matcher.getElementsByClassName("content-match-upload")[0]! as HTMLInputElement;
    content_match_upload.disabled = true;
    let match_platform:string|null = null;
    const match_core_chooser = this.content_matcher.querySelector(".core_chooser")! as HTMLSelectElement;
    match_core_chooser.onchange = (event) => {
      content_match_upload.disabled = false;
      const first_option = match_core_chooser.firstElementChild! as HTMLOptionElement;
      if (first_option.value == "(init)") {
        match_core_chooser.removeChild(first_option);
      }
      match_platform = (event.target! as HTMLInputElement).value!;
    };
    content_match_upload.onchange = (event) => {
      const files = (event.target! as HTMLInputElement).files ?? [];
      if (files.length == 1 && match_platform != null) {
        const file = files[0];
        self.clear_file_lists();
        compute_hash(file)
          .then((hash) => self.content_check_hash(match_platform!, file, hash))
          .catch((err) => {self.content_check_show_error(err)});
      }
    };
    const metadata_form = document.createElement("form");
    metadata_form.id = "work_info";
    metadata_form.innerHTML = `
<label for="work_core_chooser">Work platform:</label>
<select name="work_core_chooser" class="core_chooser">
</select>
<label for="work_name">Work name:</label>
<input name="work_name" required></input>
<label for="work_version">Version:</label>
<input name="work_version"></input>
<label for="env_config">Environment config:</label>
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
    <p class="submit-status"></p>
`;
    const core_chooser = metadata_form.getElementsByClassName("core_chooser")[0]! as HTMLSelectElement;
    this.init_core_chooser(core_chooser);
    core_chooser.onchange = () => {
      // update work platform, environment core
      const opt = core_chooser.selectedOptions[0]! as HTMLOptionElement;
      self.work.work_platform = opt.dataset["corePlatform"]!;
      self.environment.environment_platform = self.work.work_platform;
      self.environment.environment_core_name = opt.dataset["coreName"]!;
      self.environment.environment_core_version = opt.dataset["coreVersion"]!;
      self.environment.environment_framework = self.environment.environment_core_name == "v86" ? "v86" : "retroarch";
    };
    (metadata_form.querySelector('input[name="work_name"]')! as HTMLInputElement).onchange = (evt) => {
      const target = evt.target as HTMLInputElement;
      self.work.work_name = target.value;
      self.environment.environment_name = target.value;
    }
    (metadata_form.querySelector('input[name="work_version"]')! as HTMLInputElement).onchange = (evt) => {
      const target = evt.target as HTMLInputElement;
      self.work.work_version = target.value;
    }
    (metadata_form.querySelector('textarea[name="env_config"]')! as HTMLInputElement).onchange = (evt) => {
      const target = evt.target as HTMLInputElement;
      self.environment.environment_config = (target.value == "" || target.value == "null") ? {} : JSON.parse(target.value);
    }
    // other biblio stuff

    for (const input of metadata_form.querySelectorAll(".instance-file-upload") as unknown as HTMLInputElement[]) {
      const lst = input.nextElementSibling;
      const group = input.getAttribute("data-target")!;
      input.onchange = (event) => {
        console.log(event,lst);
        const files = input.files ?? [];
        if (files.length == 0) { return; }
        self.add_to_file_list(group as Role, files[0].name, {to_upload:files[0]});
      };
    }
    const button = metadata_form.querySelector("button.submit")! as HTMLButtonElement;
    const status = metadata_form.querySelector("p.submit-status")! as HTMLParagraphElement;
    button.onclick = async () => {
      let ready = true;
      // go through all files, make sure each one is uploaded or start the upload
      for (const role of Object.keys(self.file_lists)) {
        for (const [index, file] of self.file_lists[role as Role].entries()) {
          if (!('upload' in file.source)) { continue; }
          if (file.source.upload == Upload.NotStarted) {
            const upload_button = file.list_item.querySelector("button.upload")! as HTMLButtonElement;
            const progress = file.list_item.querySelector("progress")! as HTMLProgressElement;
            this.upload_file(role as Role, index, upload_button, progress);
            // TODO: set a status message somewhere
            console.log("Start uploading",role,index,file);
            ready = false;
          } else if (file.source.upload == Upload.InProgress) {
            console.log("Still uploading",role,index,file);
            ready = false;
          }
        }
      }
      if (!ready) {
        status.textContent = "Please wait until all objects have been successfully uploaded, and then click on \"Create Instance\" again";
        return;
      }
      // try to create environment and work records (but no problem if they already exist)
      status.textContent = "Creating records...";
      try {
        const work_id = (await (await fetch(`${this.base_url}/works/create`, {method:'POST', cache:'no-cache', headers:{'Content-Type':'application/json',Accept:'application/json'}, body:JSON.stringify(this.work)})).json()).work_id;
        const environment_id = (await (await fetch(`${this.base_url}/environments/create`, {method:'POST', cache:'no-cache', headers:{'Content-Type':'application/json',Accept:'application/json'}, body:JSON.stringify(this.environment)})).json()).environment_id;
        this.instance.work_id = work_id;
        this.instance.environment_id = environment_id;
        const configs = [];
        for (const conf of this.file_lists.config) {
          configs.push('existing' in conf.source ? conf.source.existing : conf.source.upload_result_id);
        }
        const dependencies = [];
        for (const dep of this.file_lists.dependency) {
          dependencies.push('existing' in dep.source ? dep.source.existing : dep.source.upload_result_id);
        }
        const content = [];
        for (const cont of this.file_lists.content) {
          content.push('existing' in cont.source ? cont.source.existing : cont.source.upload_result_id);
        }
        const instance_id = (await (await fetch(`${this.base_url}/instances/create`, {method:'POST', cache:'no-cache', headers:{'Content-Type':'application/json',Accept:'application/json'}, body:JSON.stringify({instance:this.instance, configs, dependencies, content})})).json()).instance_id;
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
    contents.appendChild(this.content_matcher);
    contents.appendChild(search_container);
    contents.appendChild(metadata_form);
    this.appendChild(contents);
  }
  async content_check_hash(platform:string, file:File, hash:string) {
    const filename = file.name;
    const match_result = this.content_matcher.getElementsByClassName("content_match_result")[0]!;
    match_result.textContent = `${filename}:${hash}`;
    const platform_esc = encodeURIComponent(platform);
    const filename_esc = encodeURIComponent(filename);
    const hash_esc = encodeURIComponent(hash);
    const core_chooser = document.querySelector("#work_info select[name=work_core_chooser]")! as HTMLSelectElement;
    const match_core_chooser = this.content_matcher.querySelector(".core_chooser")! as HTMLSelectElement;
    try {
      const resp = await (await fetch(`${this.base_url}/lookup-work?platform=${platform_esc}&filename=${filename_esc}&hash=${hash_esc}`)).json();
      // resp has work bib fields
      // and if there is an existing instance we should use that
      if (resp.instance_id) {
        this.update_work_bibinfo(resp as WorkBib);
        this.update_work_instanceenv_info(resp.instance_id);
        match_result.textContent = "Matched existing instance";
      } else {
        core_chooser.selectedIndex = match_core_chooser.selectedIndex;
        this.update_work_bibinfo(resp as WorkBib);
        match_result.textContent = "Matched existing work";
        this.add_to_file_list("content", file.name, {to_upload:file});
      }
    } catch (error) {
      console.log("No work found",error);
      core_chooser.selectedIndex = match_core_chooser.selectedIndex;
      this.update_work_bibinfo({
        work_name: filename,
        work_platform: platform,
        work_version: "",
        work_derived_from: "",
      });
      match_result.textContent = "No match found";
      this.add_to_file_list("content", file.name, {to_upload:file});
    }
  }
  content_check_show_error(err:Error) {
    console.error(err);
    this.content_matcher.getElementsByClassName("content_match_result")[0]!.textContent = `error computing hash`;
  }
  async init_core_chooser(elt:HTMLSelectElement) {
    const cores = await (await fetch(`${this.base_url}/cores`)).json();
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
  async update_work_bibinfo(work:WorkBib) {
    const core_chooser = document.querySelector("#work_info select[name=work_core_chooser]")! as HTMLSelectElement;
    const work_name = document.querySelector("#work_info input[name=work_name]")! as HTMLInputElement;
    const work_version = document.querySelector("#work_info input[name=work_version]")! as HTMLInputElement;
    // TODO: fetch other bibliographic info fields...
    if (core_chooser.selectedOptions.length == 0) {
      core_chooser.selectedIndex = Array.from(core_chooser.options).findIndex((o) => o.dataset["platform"] == work.work_platform);
    }
    work_name.value = work.work_name;
    work_version.value = work.work_version;
    this.work.work_platform = work.work_platform;
    this.work.work_name = work.work_name;
    this.work.work_version = work.work_version;
    const opt = core_chooser.selectedOptions[0]! as HTMLOptionElement;
    this.environment.environment_core_name = opt.dataset["coreName"]!;
    this.environment.environment_core_version = opt.dataset["coreVersion"]!;
    this.environment.environment_framework = this.environment.environment_core_name == "v86" ? "v86" : "retroarch";
    // TODO: set them here...
  }
  clear_file_lists() {
    this.file_lists.config.length = 0;
    this.file_lists.dependency.length = 0;
    this.file_lists.content.length = 0;
    document.querySelectorAll("#work_info .instance-file-upload + ol").forEach((lst) => {lst.innerHTML = '';});
  }
  async upload_file(role:Role, index:number, button:HTMLButtonElement, progress:HTMLProgressElement) {
    const file = this.file_lists[role][index];
    if ('existing' in file.source) { return; }
    const src = file.source as FileUpload;
    if (src.upload != Upload.NotStarted) { return; }
    src.upload = Upload.InProgress;
    button.disabled = true;
    progress.max = 100;
    try {
      const hash = await compute_hash(src.file);
      const file_id = await (new Promise((resolve,reject) => {
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
            progress.value = ((uploaded / total) * 100);
          },
          onSuccess: () => {
            const url_parts = upload.url!.split("/");
            const uuid_string = url_parts[url_parts.length - 1];
            progress.value = 100;
            src.upload_result_id = uuid_string;
            resolve(uuid_string)
          }
        });
        upload.start();
      }));
      const object_id = (await (await fetch(`${this.base_url}/objects/create`, {
        method: 'POST', cache: 'no-cache', headers: { 'Content-Type': 'application/json', Accept:'application/json'}, body:JSON.stringify({
          file_id,
          object_description:file.filename,
        })})).json()).object_id;
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
  add_to_file_list(role:Role, filename:string, source:{existing:string} | {to_upload:File}) {
    const is_existing = 'existing' in source;
    const lst = document.querySelector(`#work_info input[data-target=${role}] + ol`)! as HTMLOListElement;
    const line_item = document.createElement("li");
    const file_record = {
      ui_id:this.next_ui_id++,
      filename,
      source:is_existing ? source : {
        file:source.to_upload,
        upload:Upload.NotStarted,
        upload_progress:0,
        upload_result_id:null
      },
      list_item:line_item
    };
    line_item.innerHTML = `
${filename} <button type="button" class="move-up">^</button> <button type="button" class="move-down">v</button> <button type="button" class="remove">X</button>
`;
    (line_item.querySelector("button.move-up")! as HTMLButtonElement).onclick = (_evt) => {
      const idx = this.file_lists[role].findIndex((fr) => fr.ui_id == file_record.ui_id);
      if (idx > 0) {
        const tmp = this.file_lists[role][idx-1];
        this.file_lists[role][idx-1] = file_record;
        this.file_lists[role][idx] = tmp;
        lst.removeChild(line_item);
        lst.insertBefore(line_item, lst.children[idx-1]);
      }
    };
    (line_item.querySelector("button.move-down")! as HTMLButtonElement).onclick = (_evt) => {
      const idx = this.file_lists[role].findIndex((fr) => fr.ui_id == file_record.ui_id);
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
      const idx = this.file_lists[role].findIndex((fr) => fr.ui_id == file_record.ui_id);
      this.file_lists[role].splice(idx,1);
      lst.removeChild(line_item);
      };
    if (is_existing) {
      const exist_note = document.createElement("span");
      exist_note.classList.add("exists-note");
      exist_note.textContent = "E";
      line_item.appendChild(exist_note);
    } else {
      const upload_status = document.createElement("button");
      upload_status.classList.add("upload");
      const progress = document.createElement("progress");
      progress.max = 0;
      progress.value = 0;
      upload_status.textContent = "U";
      upload_status.onclick = (_evt) => {
        const idx = this.file_lists[role].findIndex((fr) => fr.ui_id == file_record.ui_id);
        this.upload_file(role, idx, upload_status, progress);
      };
      line_item.appendChild(upload_status);
      line_item.appendChild(progress);
    }
    lst.appendChild(line_item);
    this.file_lists[role].push(file_record);
  }
  async update_work_instanceenv_info(instance_id:string) {
    const env_config = document.querySelector("#work_info textarea[name=env_config]")! as HTMLInputElement;
    const full_instance:FullInstance = await (await fetch(`${this.base_url}/instances/${encodeURIComponent(instance_id)}`, {headers:[["Accept","application/json"]]})).json();
    env_config.value = JSON.stringify(full_instance.environment.environment_config || {});
    this.clear_file_lists();
    full_instance.objects.sort((a:ObjectLink,b:ObjectLink) => (a.object_role_index - b.object_role_index));
    for (const lnk of full_instance.objects) {
      this.add_to_file_list(lnk.object_role, lnk.file_filename, {existing:lnk.object_id});
    }
    this.environment = full_instance.environment;
    const core_chooser = document.querySelector("#work_info select[name=work_core_chooser]")! as HTMLSelectElement;
    core_chooser.selectedIndex = Array.from(core_chooser.options).findIndex((o) => 
      o.dataset["coreName"] == this.environment.environment_core_name &&
        o.dataset["corePlatform"] == this.environment.environment_platform
    );
    this.instance = full_instance.info;
    this.work = full_instance.work;
    // TODO: fetch other bibliographic info fields...
  }
}


customElements.define("gisst-new-instance", GISSTNewInstance);
