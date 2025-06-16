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
    const top_row = document.createElement("div");
    top_row.setAttribute("class", "row");
    const searchbox = document.createElement("div");
    top_row.appendChild(searchbox);
    this.appendChild(top_row);
    const bottom_row = document.createElement("div");
    bottom_row.setAttribute("class", "row");
    const resultsbox = document.createElement("div");
    const pagination = document.createElement("div");
    bottom_row.appendChild(resultsbox);
    bottom_row.appendChild(pagination);
    this.appendChild(bottom_row);

    const search = search_instances(search_url, search_key, {});
    search.addWidgets([
      widgets.searchBox({
        container: searchbox
      }),
      widgets.configure({ hitsPerPage: 10 }),
      widgets.pagination({ container: pagination }),
      widgets.hits({
        container: resultsbox,
        templates: {
          item: (hit, { html, components }) => html`
          <div>
          <div class="hit-name">
          <a href="${base_url}/instances/${hit.instance_id}">${components.Highlight({ hit, attribute: "work_name"})}</a>
          </div>
          <div class="hit-">
            ${hit.work_version}--${components.Highlight({ hit, attribute: "work_platform"})}--<a class="btn btn-primary instance-boot-button" href="${base_url}/play/${hit.instance_id}">Play</a>
          </div>
          </div>
          `
        }
      })
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
    const can_clone = (this.getAttribute("can-clone") ?? "false") === 'true';
    const filters = [];
    if (limit_to_instance && limit_to_instance != "") {
      filters.push(`instance_id = "${limit_to_instance}"`);
    }
    if (limit_to_creator && limit_to_creator != "") {
      filters.push(`creator_id = "${limit_to_creator}"`);
    }
    this.classList.add("gisst-state-search");
    const top_row = document.createElement("div");
    top_row.setAttribute("class", "row");
    const searchbox = document.createElement("div");
    top_row.appendChild(searchbox);
    this.appendChild(top_row);
    const bottom_row = document.createElement("div");
    bottom_row.setAttribute("class", "row");
    const resultsbox = document.createElement("div");
    const pagination = document.createElement("div");
    bottom_row.appendChild(resultsbox);
    bottom_row.appendChild(pagination);
    this.appendChild(bottom_row);

    const search = search_states(search_url, search_key, {});
    search.addWidgets([
      widgets.searchBox({
        container: searchbox
      }),
      widgets.configure({ hitsPerPage: 10, filters: filters.join(" AND ") }),
      widgets.pagination({ container: pagination }),
      /* TODO: these nested templates are gnarly, what can be done? */
      widgets.hits({
        container: resultsbox,
        templates: {
          item: (hit, { html, components }) => html`
          <div>
          <div class="hit-name">
          <a href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}">${components.Highlight({ hit, attribute: "state_name"})} (Play)</a>
          --
          <a href="${base_url}/play/${hit.instance_id}?state=${hit.state_id}&boot_into_record=true">(Play and record)</a>
          ${can_clone ? html`--<a class="btn btn-primary" href="${hit.instance_id}/clone?state=${ hit.state_id }">Clone as New Instance</a>` : ""}
          </div>
          <div class="hit-more-info">
            ${components.Highlight({ hit, attribute: "state_description"})}
            ${show_instance_info ? html`<br/>
            <a href="${base_url}/instances/${hit.instance_id}">${components.Highlight({ hit, attribute: "work_name"})}</a>--
            ${hit.work_version}--${components.Highlight({ hit, attribute: "work_platform"})}--<a class="btn btn-primary instance-boot-button" href="${base_url}/play/${hit.instance_id}">Play</a>` : ""}
            ${show_creator_info ? html`<br/>
            <a href="{{ base_url | safe }}/creators/${hit.creator_id}">${components.Highlight({ hit, attribute: "creator_username" })}</a>` : ""}
          </div>
        </div>`
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
    const top_row = document.createElement("div");
    top_row.setAttribute("class", "row");
    const searchbox = document.createElement("div");
    top_row.appendChild(searchbox);
    this.appendChild(top_row);
    const bottom_row = document.createElement("div");
    bottom_row.setAttribute("class", "row");
    const resultsbox = document.createElement("div");
    const pagination = document.createElement("div");
    bottom_row.appendChild(resultsbox);
    bottom_row.appendChild(pagination);
    this.appendChild(bottom_row);

    const search = search_saves(search_url, search_key, {});
    search.addWidgets([
      widgets.searchBox({
        container: searchbox
      }),
      widgets.configure({ hitsPerPage: 10, filters: filters.join(" AND ") }),
      widgets.pagination({ container: pagination }),
      /* TODO: these nested templates are gnarly, what can be done? */
      widgets.hits({
        container: resultsbox,
        templates: {
          item: (hit, { html, components }) => html`
          <div>
          <div class="hit-name">
          <a href="${base_url}/play/${hit.instance_id}?save=${hit.save_id}">${components.Highlight({ hit, attribute: "save_short_desc"})} (Play)</a>
          --
          <a href="${base_url}/play/${hit.instance_id}?save=${hit.save_id}&boot_into_record=true">(Play and record)</a>
          </div>
          <div class="hit-more-info">
            ${components.Highlight({ hit, attribute: "save_description"})}
            ${show_instance_info ? html`<br/>
            <a href="${base_url}/instances/${hit.instance_id}">${components.Highlight({ hit, attribute: "work_name"})}</a>--
            ${hit.work_version}--${components.Highlight({ hit, attribute: "work_platform"})}--<a class="btn btn-primary instance-boot-button" href="${base_url}/play/${hit.instance_id}">Play</a>` : ""}
            ${show_creator_info ? html`<br/>
            <a href="{{ base_url | safe }}/creators/${hit.creator_id}">${components.Highlight({ hit, attribute: "creator_username" })}</a>` : ""}
          </div>
        </div>`
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
    const top_row = document.createElement("div");
    top_row.setAttribute("class", "row");
    const searchbox = document.createElement("div");
    top_row.appendChild(searchbox);
    this.appendChild(top_row);
    const bottom_row = document.createElement("div");
    bottom_row.setAttribute("class", "row");
    const resultsbox = document.createElement("div");
    const pagination = document.createElement("div");
    bottom_row.appendChild(resultsbox);
    bottom_row.appendChild(pagination);
    this.appendChild(bottom_row);

    const search = search_replays(search_url, search_key, {});
    search.addWidgets([
      widgets.searchBox({
        container: searchbox
      }),
      widgets.configure({ hitsPerPage: 10, filters: filters.join(" AND ") }),
      widgets.pagination({ container: pagination }),
      /* TODO: these nested templates are gnarly, what can be done? */
      widgets.hits({
        container: resultsbox,
        templates: {
          item: (hit, { html, components }) => html`
          <div>
          <div class="hit-name">
          <a href="${base_url}/play/${hit.instance_id}?replay=${hit.replay_id}">${components.Highlight({ hit, attribute: "replay_name"})} (Play)</a>
          </div>
          <div class="hit-more-info">
            ${components.Highlight({ hit, attribute: "replay_description"})}
            ${show_instance_info ? html`<br/>
            <a href="${base_url}/instances/${hit.instance_id}">${components.Highlight({ hit, attribute: "work_name"})}</a>--
            ${hit.work_version}--${components.Highlight({ hit, attribute: "work_platform"})}--<a class="btn btn-primary instance-boot-button" href="${base_url}/play/${hit.instance_id}">Play</a>` : ""}
            ${show_creator_info ? html`<br/>
            <a href="{{ base_url | safe }}/creators/${hit.creator_id}">${components.Highlight({ hit, attribute: "creator_username" })}</a>` : ""}
          </div>
        </div>`
        }
      })
    ]);
    search.start();
  }
}

customElements.define("gisst-performance-search", GISSTPerformanceSearch);
