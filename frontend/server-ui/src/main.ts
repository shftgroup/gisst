import { instantMeiliSearch, InstantMeiliSearchOptions } from '@meilisearch/instant-meilisearch';
import { default as instantsearch, InstantSearch } from 'instantsearch.js';
import 'instantsearch.css/themes/reset.css';
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

class GISSTInstanceSearch extends HTMLElement {
  constructor() {
    super();
  }
  connectedCallback() {
    const search_url = this.getAttribute("search_url");
    const search_key = this.getAttribute("search_key");
    const base_url = this.getAttribute("base_url");
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
