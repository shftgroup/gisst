import {
  type TemplateWithBindEvent as Template,
  type Hit,
} from "instantsearch.js";
import { search_instances } from "./search_impl";
import * as scs from "./search_components";
import { type InstanceRecord } from "./search_components";
import { GISSTItemSearch, type SearchFunction } from "./item_search";

export class GISSTInstanceSearch extends GISSTItemSearch {
  constructor() {
    super();
  }
  modeltype(): string {
    return "instance";
  }
  search_function(): SearchFunction {
    return search_instances;
  }
  item_id(hit: InstanceRecord): string {
    return hit.instance_id;
  }
  short_title_key(): string {
    return "work_name";
  }
  description_key(): string {
    return "work_platform • work_version";
  }
  item_template(): Template<Hit<InstanceRecord>> {
    return (hit: Hit<InstanceRecord>, { html, components }) => html`
      <div class="gisst-Search-results-row">
        ${scs.instance_info_3col(this.base_url, hit, html, components)}
        <div class="gisst-Search-cell gisst-Search-actions-cell">
          ${scs.play_actions(this.base_url, hit.instance_id, hit.instance_id, "instance", false, html)}
        </div>
      </div>
    `;
  }
}
