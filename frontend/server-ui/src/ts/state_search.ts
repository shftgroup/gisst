import { GISSTCreatorItemSearch } from "./creator_item_search";
import { type SearchFunction } from "./item_search";
import { search_states } from "./search_impl";
import { type CreatorRecord } from "./search_components";

interface StateRecord extends CreatorRecord {
  state_id: string;
  screenshot: string;
  state_name: string;
  state_description: string;
}

export class GISSTStateSearch extends GISSTCreatorItemSearch {
  constructor() {
    super();
  }
  modeltype(): string {
    return "state";
  }
  search_function(): SearchFunction {
    return search_states;
  }
  item_id(hit: CreatorRecord): string {
    return (hit as StateRecord).state_id;
  }
  short_title_key(): string {
    return "state_name";
  }
  description_key(): string {
    return "state_description";
  }
  screenshot(hit: StateRecord): string | null {
    return hit.screenshot;
  }
  can_clone(hit: StateRecord): boolean {
    return hit.environment_framework == "v86" && this.active_creator_role <= 50;
  }
  connectedCallback() {
    super.connectedCallback();
  }
}
