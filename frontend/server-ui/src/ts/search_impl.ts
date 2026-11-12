import { instantMeiliSearch } from "@meilisearch/instant-meilisearch";
import { InstantMeiliSearchOptions } from "@meilisearch/instant-meilisearch";
import { default as instantsearch, InstantSearch } from "instantsearch.js";
export type SearchObject = InstantSearch;
export type SearchOptions = InstantMeiliSearchOptions;
export function search_instances(
  search_host: string,
  search_key: string,
  params: InstantMeiliSearchOptions,
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
  params: InstantMeiliSearchOptions,
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
  params: InstantMeiliSearchOptions,
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
  params: InstantMeiliSearchOptions,
): InstantSearch {
  params.primaryKey = "replay_id";
  const { searchClient } = instantMeiliSearch(search_host, search_key, params);
  return instantsearch({
    indexName: "replay",
    searchClient,
  });
}
