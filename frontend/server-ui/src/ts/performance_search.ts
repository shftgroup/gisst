import { GISSTCreatorItemSearch } from "./creator_item_search";
import { type SearchFunction } from "./item_search";
import { search_replays } from "./search_impl";
import { type CreatorRecord } from "./search_components";

interface PerformanceRecord extends CreatorRecord {
  replay_id: string;
  replay_name: string;
  replay_description: string;
}

export class GISSTPerformanceSearch extends GISSTCreatorItemSearch {
  constructor() {
    super();
  }
  modeltype(): string {
    return "replay";
  }
  search_function(): SearchFunction {
    return search_replays;
  }
  item_id(hit: CreatorRecord): string {
    return (hit as PerformanceRecord).replay_id;
  }
  short_title_key(): string {
    return "replay_name";
  }
  description_key(): string {
    return "replay_description";
  }
  connectedCallback() {
    super.connectedCallback();
  }
}
