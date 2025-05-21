import { instantMeiliSearch, InstantMeiliSearchOptions } from '@meilisearch/instant-meilisearch';
import { default as instantsearch, InstantSearch } from 'instantsearch.js';
export { default as styles } from 'instantsearch.css/themes/reset.css?inline';

export type SearchOptions=InstantMeiliSearchOptions;

export * as search from 'instantsearch.js';
export * as widgets from 'instantsearch.js/es/widgets';

export function search_instances(search_host:string, search_key:string, params:SearchOptions):InstantSearch {
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

