import { GISSTNewInstance } from "./new_instance";
import { GISSTInstanceSearch } from "./instance_search";
import { GISSTStateSearch } from "./state_search";
import { GISSTSaveSearch } from "./save_search";
import { GISSTPerformanceSearch } from "./performance_search";
import { GISSTNewInstanceEnvFormRA } from "./env-forms/ra";
import { GISSTNewInstanceEnvFormV86 } from "./env-forms/v86";

import "instantsearch.css/themes/reset.css";
import "gisst-style/base.css";
import "../css/server-ui-main.css";
import "../css/server-ui-new-instance.css";
import "../css/server-ui-search.css";
import "../css/server-ui-instance.css";

customElements.define("gisst-instance-search", GISSTInstanceSearch);
customElements.define("gisst-state-search", GISSTStateSearch);
customElements.define("gisst-save-search", GISSTSaveSearch);
customElements.define("gisst-performance-search", GISSTPerformanceSearch);
customElements.define(
  "gisst-new-instance-env-form-v86",
  GISSTNewInstanceEnvFormV86,
);
customElements.define(
  "gisst-new-instance-env-form-ra",
  GISSTNewInstanceEnvFormRA,
);
customElements.define("gisst-new-instance", GISSTNewInstance);
