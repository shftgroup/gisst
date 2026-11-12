import { GISSTCreatorItemSearch } from "./creator_item_search";
import { type SearchFunction } from "./item_search";
import { search_saves } from "./search_impl";
import { type CreatorRecord } from "./search_components";

interface SaveRecord extends CreatorRecord {
  save_id: string;
  save_short_desc: string;
  save_description: string;
}

export class GISSTSaveSearch extends GISSTCreatorItemSearch {
  constructor() {
    super();
  }
  modeltype(): string {
    return "save";
  }
  search_function(): SearchFunction {
    return search_saves;
  }
  item_id(hit: CreatorRecord): string {
    return (hit as SaveRecord).save_id;
  }
  short_title_key(): string {
    return "save_short_desc";
  }
  description_key(): string {
    return "save_description";
  }
  connectedCallback() {
    super.connectedCallback();
  }
}
