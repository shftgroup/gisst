import * as widgets from "instantsearch.js/es/widgets";
import * as tus from "tus-js-client";
import { search_instances } from "./main";
import { default as SparkMD5 } from "spark-md5";
import connectStats from "instantsearch.js/es/connectors/stats/connectStats";
import { elementFromTemplates } from "./main";
import { GISSTNewInstanceEnvForm } from "./env-forms/common";
import { GISSTNewInstanceEnvFormV86 } from "./env-forms/v86";
import { GISSTNewInstanceEnvFormRA } from "./env-forms/ra";

interface WorkBib {
  work_name: string;
  work_platform: string;
  work_version: string;
  work_derived_from: string;
}

interface Work extends WorkBib {
  work_id: string;
}

interface InstanceWork extends Work {
  instance_id: string;
  environment_id: string;
}

interface Instance {
  instance_id: string;
  work_id: string;
  environment_id: string;
  instance_config: object | null;
  created_on: Date;
  derived_from_instance: string;
  derived_from_state: string;
}

interface Environment {
  environment_id: string;
  environment_name: string;
  environment_platform: string;
  environment_framework: string;
  environment_core_name: string;
  environment_core_version: string;
  environment_derived_from: string;
  environment_config: object | null;
  created_on: Date;
}

type Role = "content" | "dependency" | "config";

interface ObjectLink {
  object_id: string;
  object_role: Role;
  object_role_index: number;
  file_hash: string;
  file_filename: string;
  file_source_path: string;
  file_dest_path: string;
}

interface FullInstance {
  work: Work;
  info: Instance;
  environment: Environment;
  objects: ObjectLink[];
}

enum Upload {
  NotStarted,
  InProgress,
  Finished,
}

interface ExistingFile {
  existing: string;
}

interface FileUpload {
  file: File;
  upload: Upload;
  upload_result_id: string | null;
}

interface InstanceFile {
  ui_id: number;
  filename: string;
  list_item: HTMLLIElement;
  source: ExistingFile | FileUpload;
}

interface CoreInfo {
  core_name: string;
  core_version: string;
  core_platform: string;
}

function compute_hash(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const chunkSize = 16 * 1024 * 1024,
      chunks = Math.ceil(file.size / chunkSize),
      spark = new SparkMD5.ArrayBuffer(),
      fileReader = new FileReader();
    let currentChunk = 0;
    fileReader.onload = function (e) {
      spark.append(e.target!.result as ArrayBuffer);
      currentChunk++;

      if (currentChunk < chunks) {
        loadNext();
      } else {
        resolve(spark.end());
      }
    };
    fileReader.onerror = function () {
      reject(new Error("Hash computation error", { cause: fileReader.error }));
    };
    function loadNext() {
      const start = currentChunk * chunkSize,
        end = start + chunkSize >= file.size ? file.size : start + chunkSize;
      fileReader.readAsArrayBuffer(file.slice(start, end));
    }
    loadNext();
  });
}

export class GISSTNewInstance extends HTMLElement {
  content_matcher: HTMLDivElement;
  base_url: string;
  next_ui_id: number;
  file_lists: {
    config: InstanceFile[];
    dependency: InstanceFile[];
    content: InstanceFile[];
  } = {
    config: [],
    dependency: [],
    content: [],
  };
  work: Work;
  instance: Instance;
  environment: Environment;
  env_form_active: GISSTNewInstanceEnvForm | null;
  env_form_v86: GISSTNewInstanceEnvFormV86 | null;
  env_form_ra: GISSTNewInstanceEnvFormRA | null;
  env_form_mode: "form" | "manual";

  constructor() {
    super();

    this.work = {
      work_id: "",
      work_name: "",
      work_version: "",
      work_platform: "",
      work_derived_from: "",
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
      created_on: new Date(),
    };
    this.base_url = "/";
    this.next_ui_id = 0;
    this.content_matcher = document.createElement("div");
    this.env_form_mode = "form";
    this.env_form_active = null;
    this.env_form_v86 = null;
    this.env_form_ra = null;
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

    if (
      search_url == undefined ||
      search_key == undefined ||
      base_url == undefined
    ) {
      throw new Error(
        "Cannot create instance search UI without search url, search key, and base url",
      );
    }

    this.base_url = base_url;

    // Initialize templates
    this.content_matcher = elementFromTemplates("new-instance-method-two")
      .firstElementChild as HTMLDivElement;
    const search_method_div = elementFromTemplates("new-instance-method-one")
      .firstElementChild as HTMLDivElement;
    const metadata_form = elementFromTemplates("new-instance-details-form")
      .firstElementChild as HTMLFormElement;
    const select_files_form = elementFromTemplates(
      "new-instance-select-files-form",
    ).firstElementChild as HTMLFormElement;
    const submit_div = elementFromTemplates("new-instance-submit")
      .firstElementChild as HTMLDivElement;

    // config panels for form/manual entry tabs
    const config_form = elementFromTemplates("new-instance-config-form")
      .firstElementChild as HTMLDivElement;
    const missing_config_form_panel = elementFromTemplates(
      "missing-config-form-panel",
    ).firstElementChild as HTMLDivElement;
    const config_manual_panel = elementFromTemplates(
      "config-manual-entry-panel",
    ).firstElementChild as HTMLDivElement;

    // Setup method 1 search
    const search_box = search_method_div.querySelector(
      "#new-instance-search-box",
    ) as HTMLElement;
    const results_box = search_method_div.querySelector(
      "#new-instance-results-box",
    ) as HTMLElement;
    const results_header = search_method_div.querySelector(
      ".gisst-New-instance-search-header",
    ) as HTMLElement;
    const results_header_display = results_header.style.display;

    const search = search_instances(search_url, search_key, {});

    // Custom handler for hiding and showing table header
    const custom_stats = connectStats((renderOptions) => {
      if (renderOptions.nbHits <= 0 || !renderOptions.query) {
        results_header.style.display = "none";
      } else {
        results_header.style.display = results_header_display;
      }
    });

    search.addWidgets([
      widgets.searchBox({
        container: search_box,
      }),
      widgets.hits({
        container: results_box,
        transformItems(items, { results }) {
          // Don't show any results for empty query
          if (!results?.query) return [];
          return items;
        },
        templates: {
          item(hit, { html, components }) {
            return html`
              <button class="gisst-New-instance-search-item-button gisst-Search-results-row" onClick="${() => {
                console.log("selected ", hit);

                const work = hit as object as InstanceWork;
                self.update_work_bibinfo(work);
                void self.update_work_instanceenv_info(work.instance_id);

                // Set the search bar to the name of the work without showing any results
                search.helper!.setQuery("").search();
                search.helper!.setQuery(work.work_name);

                if (document.activeElement) {
                  (document.activeElement as HTMLElement).blur();
                }
              }}">
                <div
                  class="gisst-Search-cell gisst-New-instance-search-work-name"
                >
                  ${components.Highlight({ hit, attribute: "work_name" })}</span
                </div>
                <div
                  class="gisst-Search-cell gisst-New-instance-search-platform-info"
                >
                  ${components.Highlight({ hit, attribute: "work_platform" })}
                </div>
                <div
                  class="gisst-Search-cell gisst-New-instance-search-version-info"
                >
                  ${hit.work_version}
                </div>
              </button>
            `;
          },
          empty() {
            return null;
          },
        },
      }),
      widgets.configure({ hitsPerPage: 10 }),
      custom_stats({}),
    ]);

    search.start();

    // Setup method 2
    void this.init_core_chooser(
      this.content_matcher.getElementsByClassName(
        "core_chooser",
      )[0] as HTMLSelectElement,
    );

    /* eslint @typescript-eslint/no-this-alias: "off" */
    const self = this;

    const content_match_upload = this.content_matcher.getElementsByClassName(
      "content-match-upload",
    )[0] as HTMLInputElement;
    content_match_upload.disabled = true;
    let match_platform: string | null = null;
    const match_core_chooser: HTMLSelectElement =
      this.content_matcher.querySelector(".core_chooser")!;

    // Method 2 Event Listeners
    match_core_chooser.onchange = (event) => {
      content_match_upload.disabled = false;
      const first_option =
        match_core_chooser.firstElementChild! as HTMLOptionElement;
      if (first_option.value == "(init)") {
        match_core_chooser.removeChild(first_option);
      }
      match_platform = (event.target! as HTMLInputElement).value!;
    };

    const content_file_name =
      this.content_matcher.querySelector(".content-file-name");
    content_match_upload.onchange = (event) => {
      const files = (event.target! as HTMLInputElement).files ?? [];
      if (files.length == 1 && match_platform != null) {
        const file = files[0];
        self.clear_file_lists();
        compute_hash(file)
          .then((hash) => self.content_check_hash(match_platform!, file, hash))
          .catch((err) => {
            self.content_check_show_error(
              new Error("content hash check failed", { cause: err }),
            );
          });
        if (content_file_name != null) {
          content_file_name.innerHTML = file.name;
        }
      }
    };

    const core_chooser = metadata_form.getElementsByClassName(
      "core_chooser",
    )[0] as HTMLSelectElement;
    void this.init_core_chooser(core_chooser);
    core_chooser.onchange = () => {
      // update work platform, environment core
      const opt = core_chooser.selectedOptions[0];
      self.work.work_platform = opt.dataset["corePlatform"]!;
      self.environment.environment_platform = self.work.work_platform;
      self.environment.environment_core_name = opt.dataset["coreName"]!;
      self.environment.environment_core_version = opt.dataset["coreVersion"]!;
      self.environment.environment_framework =
        self.environment.environment_core_name == "v86" ? "v86" : "retroarch";
      this.clear_file_lists();
      this.env_form_state_sync();
      this.env_form_reset();
    };

    metadata_form.querySelector<HTMLInputElement>(
      'input[name="work_name"]',
    )!.onchange = (evt) => {
      const target = evt.target as HTMLInputElement;
      self.work.work_name = target.value;
      self.environment.environment_name = target.value;
    };

    metadata_form.querySelector<HTMLInputElement>(
      'input[name="work_version"]',
    )!.onchange = (evt) => {
      const target = evt.target as HTMLInputElement;
      self.work.work_version = target.value;
    };

    config_manual_panel.querySelector<HTMLInputElement>(
      'textarea[name="env_config"]',
    )!.onchange = (evt) => {
      const target = evt.target as HTMLInputElement;
      self.environment.environment_config =
        target.value == "" || target.value == "null"
          ? {}
          : (JSON.parse(target.value) as object);
    };

    // other biblio stuff

    for (const input of select_files_form.querySelectorAll(
      ".instance-file-upload",
    ) as unknown as HTMLInputElement[]) {
      const group = input.getAttribute("data-target")!;
      input.onchange = (event) => {
        console.log(event);
        const files = input.files ?? [];
        if (files.length == 0) {
          return;
        }
        self.add_to_file_list(group as Role, files[0].name, {
          to_upload: files[0],
        });
      };
    }

    // initialize config method tabbing
    const tabsContainer = config_form.querySelector(".gisst-New-instance-tabs");
    const tabList = tabsContainer!.querySelector('[role="tablist"]');
    const tabs = Array.from(
      tabList!.querySelectorAll<HTMLButtonElement>('[role="tab"]'),
    );
    const tabPanelsContainer = tabsContainer!.querySelector(".tab-panels");
    const tabPanels = Array.from(
      tabPanelsContainer!.querySelectorAll<HTMLDivElement>('[role="tabpanel"]'),
    );

    const panel_1 = tabPanelsContainer!.querySelector(
      "#panel-1",
    ) as HTMLDivElement;
    const panel_2 = tabPanelsContainer!.querySelector(
      "#panel-2",
    ) as HTMLDivElement;
    this.env_form_v86 = new GISSTNewInstanceEnvFormV86();
    this.env_form_ra = new GISSTNewInstanceEnvFormRA();

    panel_1.appendChild(missing_config_form_panel);
    panel_1.appendChild(this.env_form_v86);
    panel_1.appendChild(this.env_form_ra);
    panel_2.appendChild(config_manual_panel);

    // switch panel visibility
    tabs.forEach((tab) => {
      tab.addEventListener("click", (e) => {
        const btn = e.target as HTMLButtonElement;
        this.showTab(btn, tabs, tabPanels);

        if (btn.dataset.mode === "form") {
          this.env_form_mode = "form";
          this.clear_file_lists();
        } else {
          this.env_form_mode = "manual";
        }
        this.env_form_state_sync();
        this.env_form_reset();
      });
    });

    // Environment Config Form Handlers
    this.addEventListener("gisst-env-form-on-new-conf", (ev) => {
      this.environment.environment_config = ev.new_conf;
    });
    this.addEventListener("gisst-env-form-on-content-add", (ev) => {
      this.add_to_file_list(
        "content",
        ev.content_file.name,
        {
          to_upload: ev.content_file,
        },
        ev.index,
      );
    });
    this.addEventListener("gisst-env-form-on-content-remove", (ev) => {
      this.remove_from_file_list("content", ev.index);
    });

    // Form submission handling
    const button =
      submit_div.querySelector<HTMLButtonElement>("button.submit")!;
    const status =
      submit_div.querySelector<HTMLParagraphElement>("p.submit-status")!;

    button.onclick = async () => {
      let ready = true;

      // go through all files, make sure each one is uploaded or start the upload
      for (const role of Object.keys(self.file_lists)) {
        for (const [index, file] of self.file_lists[role as Role].entries()) {
          if (!("upload" in file.source)) {
            continue;
          }

          if (file.source.upload == Upload.NotStarted) {
            const upload_button =
              file.list_item.querySelector<HTMLButtonElement>("button.upload")!;
            const progress =
              file.list_item.querySelector<HTMLProgressElement>("progress")!;

            void this.upload_file(role as Role, index, upload_button, progress);
            // TODO: set a status message somewhere
            console.log("Start uploading", role, index, file);
            ready = false;
          } else if (file.source.upload == Upload.InProgress) {
            console.log("Still uploading", role, index, file);
            ready = false;
          }
        }
      }

      if (!ready) {
        status.textContent =
          'Please wait until all objects have been successfully uploaded, and then click on "Create Instance" again';
        return;
      }

      // try to create environment and work records (but no problem if they already exist)
      status.textContent = "Creating records...";
      try {
        const work_id = (
          (await (
            await fetch(`${this.base_url}/works/create`, {
              method: "POST",
              cache: "no-cache",
              headers: {
                "Content-Type": "application/json",
                Accept: "application/json",
              },
              body: JSON.stringify(this.work),
            })
          ).json()) as Work
        ).work_id;

        const environment_id = (
          (await (
            await fetch(`${this.base_url}/environments/create`, {
              method: "POST",
              cache: "no-cache",
              headers: {
                "Content-Type": "application/json",
                Accept: "application/json",
              },
              body: JSON.stringify(this.environment),
            })
          ).json()) as Environment
        ).environment_id;

        this.instance.work_id = work_id;
        this.instance.environment_id = environment_id;
        const configs = [];
        for (const conf of this.file_lists.config) {
          configs.push(
            "existing" in conf.source
              ? conf.source.existing
              : conf.source.upload_result_id,
          );
        }

        const dependencies = [];
        for (const dep of this.file_lists.dependency) {
          dependencies.push(
            "existing" in dep.source
              ? dep.source.existing
              : dep.source.upload_result_id,
          );
        }

        const content = [];
        for (const cont of this.file_lists.content) {
          content.push(
            "existing" in cont.source
              ? cont.source.existing
              : cont.source.upload_result_id,
          );
        }

        const instance_id = (
          (await (
            await fetch(`${this.base_url}/instances/create`, {
              method: "POST",
              cache: "no-cache",
              headers: {
                "Content-Type": "application/json",
                Accept: "application/json",
              },
              body: JSON.stringify({
                instance: this.instance,
                configs,
                dependencies,
                content,
              }),
            })
          ).json()) as Instance
        ).instance_id;

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
    contents.className = "gisst-New-instance-container";
    contents.appendChild(elementFromTemplates("new-instance-header"));
    contents.appendChild(search_method_div);
    contents.appendChild(this.content_matcher);
    contents.appendChild(metadata_form);
    contents.appendChild(config_form);
    contents.appendChild(select_files_form);
    contents.appendChild(submit_div);
    this.appendChild(contents);

    this.env_form_state_sync();
    this.env_form_reset();
  }

  async content_check_hash(platform: string, file: File, hash: string) {
    const filename = file.name;
    const match_result = this.content_matcher.getElementsByClassName(
      "content_match_result",
    )[0];
    match_result.textContent = `${filename}:${hash}`;
    const platform_esc = encodeURIComponent(platform);
    const filename_esc = encodeURIComponent(filename);
    const hash_esc = encodeURIComponent(hash);
    const core_chooser = document.querySelector<HTMLSelectElement>(
      "#work-info select[name=work_core_chooser]",
    )!;
    const match_core_chooser =
      this.content_matcher.querySelector<HTMLSelectElement>(".core_chooser")!;
    try {
      const resp = (await (
        await fetch(
          `${this.base_url}/lookup-work?platform=${platform_esc}&filename=${filename_esc}&md5=${hash_esc}`,
        )
      ).json()) as WorkBib;
      // resp has work bib fields
      // and if there is an existing instance we should use that
      if ("instance_id" in resp) {
        this.update_work_bibinfo(resp);
        await this.update_work_instanceenv_info(
          (resp as InstanceWork).instance_id,
        );
        match_result.textContent = "Matched existing instance";
      } else {
        core_chooser.selectedIndex = match_core_chooser.selectedIndex;
        this.update_work_bibinfo(resp);
        match_result.textContent = "Matched known work";
        this.add_to_file_list("content", file.name, { to_upload: file });
      }
    } catch (error) {
      console.log("No work found", error);
      core_chooser.selectedIndex = match_core_chooser.selectedIndex;
      this.update_work_bibinfo({
        work_name: filename,
        work_platform: platform,
        work_version: "",
        work_derived_from: "",
      });
      match_result.textContent = "No match found";
      this.add_to_file_list("content", file.name, { to_upload: file });
    }
  }

  content_check_show_error(err: Error) {
    console.error(err);
    this.content_matcher.getElementsByClassName(
      "content_match_result",
    )[0].textContent = `error computing hash`;
  }

  async init_core_chooser(elt: HTMLSelectElement) {
    const cores = (await (await fetch(`${this.base_url}/cores`)).json()) as {
      cores: CoreInfo[];
    };
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

  update_work_bibinfo(work: WorkBib) {
    const core_chooser = document.querySelector<HTMLSelectElement>(
      "#work-info select[name=work_core_chooser]",
    )!;
    const work_name = document.querySelector<HTMLInputElement>(
      "#work-info input[name=work_name]",
    )!;
    const work_version = document.querySelector<HTMLInputElement>(
      "#work-info input[name=work_version]",
    )!;
    // TODO: fetch other bibliographic info fields...
    if (core_chooser.selectedOptions.length == 0) {
      core_chooser.selectedIndex = Array.from(core_chooser.options).findIndex(
        (o) => o.dataset["platform"] == work.work_platform,
      );
    }
    work_name.value = work.work_name;
    work_version.value = work.work_version;
    this.work.work_platform = work.work_platform;
    this.work.work_name = work.work_name;
    this.work.work_version = work.work_version;
    const opt = core_chooser.selectedOptions[0];
    this.environment.environment_core_name = opt.dataset["coreName"]!;
    this.environment.environment_core_version = opt.dataset["coreVersion"]!;
    this.environment.environment_platform = this.work.work_platform;
    this.environment.environment_framework =
      this.environment.environment_core_name == "v86" ? "v86" : "retroarch";
    // TODO: set them here...
  }

  clear_file_lists() {
    this.file_lists.config.length = 0;
    this.file_lists.dependency.length = 0;
    this.file_lists.content.length = 0;

    document
      .querySelectorAll("#work-files .gisst-New-instance-select-file-lists")
      .forEach((lst) => {
        lst.innerHTML = "";
      });
  }

  async upload_file(
    role: Role,
    index: number,
    button: HTMLButtonElement,
    progress: HTMLProgressElement,
  ) {
    const file = this.file_lists[role][index];
    if ("existing" in file.source) {
      return;
    }
    const src = file.source;
    if (src.upload != Upload.NotStarted) {
      return;
    }
    src.upload = Upload.InProgress;
    button.disabled = true;
    progress.max = 100;
    try {
      const hash = await compute_hash(src.file);
      const file_id = await new Promise((resolve, reject) => {
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
            progress.value = (uploaded / total) * 100;
          },
          onSuccess: () => {
            const url_parts = upload.url!.split("/");
            const uuid_string = url_parts[url_parts.length - 1];
            progress.value = 100;
            src.upload_result_id = uuid_string;
            resolve(uuid_string);
          },
        });
        upload.start();
      });
      const object_id = (
        (await (
          await fetch(`${this.base_url}/objects/create`, {
            method: "POST",
            cache: "no-cache",
            headers: {
              "Content-Type": "application/json",
              Accept: "application/json",
            },
            body: JSON.stringify({
              file_id,
              object_description: file.filename,
            }),
          })
        ).json()) as ObjectLink
      ).object_id;
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

  /**
   * Updates whether the movement buttons should show on the
   * content files based on the current state of the form.
   */
  file_list_update_move() {
    for (const file of this.file_lists.content) {
      const move_up = file.list_item.querySelector(
        ".gisst-New-instance-move-up",
      ) as HTMLElement;
      const move_down = file.list_item.querySelector(
        ".gisst-New-instance-move-down",
      ) as HTMLElement;
      const allow_move = !this.env_form_active?.HANDLES_CONTENT_FILES;
      move_up.style.visibility = allow_move ? "" : "hidden";
      move_down.style.visibility = allow_move ? "" : "hidden";
    }
  }

  /**
   * Sync the state of the environment config form with the selected platform and form mode.
   */
  env_form_state_sync() {
    // Show correct form based on page state
    const missing_form_body = document.querySelector(
      "#missing-env-form-body",
    ) as HTMLElement;
    const is_form_mode = this.env_form_mode === "form";
    switch (this.environment.environment_framework) {
      case "v86":
        this.env_form_v86!.style.display = "";
        missing_form_body.style.display = "none";
        this.env_form_ra!.style.display = "none";
        this.env_form_active = is_form_mode ? this.env_form_v86 : null;
        break;

      case "retroarch":
        this.env_form_v86!.style.display = "none";
        missing_form_body.style.display = "none";
        this.env_form_ra!.style.display = "";
        this.env_form_active = is_form_mode ? this.env_form_ra : null;
        break;

      default:
        this.env_form_v86!.style.display = "none";
        missing_form_body.style.display = "";
        this.env_form_ra!.style.display = "none";
        this.env_form_active = null;
        break;
    }

    const manual_text_area_value =
      document.querySelector<HTMLTextAreaElement>("#env-config-manual")?.value;
    const parsed_manual =
      !manual_text_area_value || manual_text_area_value === "null"
        ? {}
        : (JSON.parse(manual_text_area_value) as object);
    this.environment.environment_config =
      this.env_form_active?.get_env_conf() ?? parsed_manual;

    // Disable content file upload if form handles it
    const content_file_upload = document.querySelector(
      "#instance-content-upload",
    ) as HTMLInputElement;
    content_file_upload.disabled =
      this.env_form_active?.HANDLES_CONTENT_FILES ?? false;
    this.file_list_update_move();
  }

  /**
   * Reset the environment form that is active.
   */
  env_form_reset() {
    if (this.env_form_active) {
      this.env_form_active.reset();
      this.environment.environment_config = {};
    }
  }

  add_to_file_list(
    role: Role,
    filename: string,
    source: { existing: string } | { to_upload: File },
    index?: number,
  ) {
    const is_existing = "existing" in source;
    const lst = document.querySelector<HTMLOListElement>(
      `#work-files fieldset.gisst-New-instance-file-selects:has(input[data-target=${role}]) ol.gisst-New-instance-select-file-lists`,
    )!;
    const line_item = document.createElement("li");
    const file_record = {
      ui_id: this.next_ui_id++,
      filename,
      source: is_existing
        ? source
        : {
            file: source.to_upload,
            upload: Upload.NotStarted,
            upload_progress: 0,
            upload_result_id: null,
          },
      list_item: line_item,
    };

    line_item.innerHTML = `
      <div class="gisst-New-instance-line-item-top">${filename} <button type="button" class="remove gisst-New-instance-remove-btn gisst-New-instance-line-item-buttons" title="Remove">
        <svg class="ais-SearchBox-resetIcon" viewBox="0 0 20 20" width="10" height="10" aria-hidden="true">
          <path d="M8.114 10L.944 2.83 0 1.885 1.886 0l.943.943L10 8.113l7.17-7.17.944-.943L20 1.886l-.943.943-7.17 7.17 7.17 7.17.943.944L18.114 20l-.943-.943-7.17-7.17-7.17 7.17-.944.943L0 18.114l.943-.943L8.113 10z"></path>
        </svg></button></div>
    `;

    const line_item_bottom_div = document.createElement("div");
    line_item_bottom_div.classList.add("gisst-New-instance-line-item-bottom");
    line_item.appendChild(line_item_bottom_div);

    const line_item_upload_div = document.createElement("div");
    line_item_bottom_div.appendChild(line_item_upload_div);

    const line_item_move_div = document.createElement("div");
    line_item_bottom_div.appendChild(line_item_move_div);

    if (is_existing) {
      const exist_note = document.createElement("span");
      exist_note.classList.add("exists-note");
      exist_note.textContent = "Existing file";
      line_item_upload_div.appendChild(exist_note);
    } else {
      // Upload button and progress bar
      const upload_status = document.createElement("button");
      upload_status.classList.add("gisst-New-instance-upload-button");
      upload_status.classList.add("gisst-New-instance-line-item-buttons");
      upload_status.title = "Upload";
      upload_status.innerHTML = `
	      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-cloud-upload" viewBox="0 0 16 16">
	        <path fill-rule="evenodd" d="M4.406 1.342A5.53 5.53 0 0 1 8 0c2.69 0 4.923 2 5.166 4.579C14.758 4.804 16 6.137 16 7.773 16 9.569 14.502 11 12.687 11H10a.5.5 0 0 1 0-1h2.688C13.979 10 15 8.988 15 7.773c0-1.216-1.02-2.228-2.313-2.228h-.5v-.5C12.188 2.825 10.328 1 8 1a4.53 4.53 0 0 0-2.941 1.1c-.757.652-1.153 1.438-1.153 2.055v.448l-.445.049C2.064 4.805 1 5.952 1 7.318 1 8.785 2.23 10 3.781 10H6a.5.5 0 0 1 0 1H3.781C1.708 11 0 9.366 0 7.318c0-1.763 1.266-3.223 2.942-3.593.143-.863.698-1.723 1.464-2.383z"></path>
	        <path fill-rule="evenodd" d="M7.646 4.146a.5.5 0 0 1 .708 0l3 3a.5.5 0 0 1-.708.708L8.5 5.707V14.5a.5.5 0 0 1-1 0V5.707L5.354 7.854a.5.5 0 1 1-.708-.708l3-3z"></path>
	      </svg>
	      `;
      upload_status.onclick = (_evt) => {
        const idx = this.file_lists[role].findIndex(
          (fr) => fr.ui_id == file_record.ui_id,
        );
        void this.upload_file(role, idx, upload_status, progress);
      };

      const progress = document.createElement("progress");
      progress.max = 0;
      progress.value = 0;

      line_item_upload_div.appendChild(upload_status);
      line_item_upload_div.appendChild(progress);
    }

    const can_move = !this.env_form_active?.HANDLES_CONTENT_FILES;
    const move_up_btn = document.createElement("button");
    move_up_btn.type = "button";
    move_up_btn.classList.add("gisst-New-instance-move-up");
    move_up_btn.classList.add("gisst-New-instance-line-item-buttons");
    move_up_btn.textContent = "↑";
    move_up_btn.title = "Move up";
    move_up_btn.style.visibility = can_move ? "" : "hidden";
    line_item_move_div.appendChild(move_up_btn);

    const move_down_btn = document.createElement("button");
    move_down_btn.type = "button";
    move_down_btn.classList.add("gisst-New-instance-move-down");
    move_down_btn.classList.add("gisst-New-instance-line-item-buttons");
    move_down_btn.textContent = "↓";
    move_down_btn.title = "Move down";
    move_down_btn.style.visibility = can_move ? "" : "hidden";
    line_item_move_div.appendChild(move_down_btn);

    line_item.querySelector<HTMLButtonElement>(
      "button.gisst-New-instance-move-up",
    )!.onclick = (_evt) => {
      const idx = this.file_lists[role].findIndex(
        (fr) => fr.ui_id == file_record.ui_id,
      );
      if (idx > 0) {
        const tmp = this.file_lists[role][idx - 1];
        this.file_lists[role][idx - 1] = file_record;
        this.file_lists[role][idx] = tmp;

        lst.removeChild(line_item);
        lst.insertBefore(line_item, lst.children[idx - 1]);
      }
    };

    line_item.querySelector<HTMLButtonElement>(
      "button.gisst-New-instance-move-down",
    )!.onclick = (_evt) => {
      const idx = this.file_lists[role].findIndex(
        (fr) => fr.ui_id == file_record.ui_id,
      );
      if (idx < this.file_lists[role].length - 1) {
        const tmp = this.file_lists[role][idx + 1];
        this.file_lists[role][idx + 1] = file_record;
        this.file_lists[role][idx] = tmp;

        const swap_item = lst.children.item(idx + 1)!;
        lst.removeChild(swap_item);
        lst.insertBefore(swap_item, line_item);
      }
    };
    line_item.querySelector<HTMLButtonElement>("button.remove")!.onclick = (
      _evt,
    ) => {
      const idx = this.file_lists[role].findIndex(
        (fr) => fr.ui_id == file_record.ui_id,
      );

      this.file_lists[role].splice(idx, 1);
      lst.removeChild(line_item);

      if (role === "content") {
        this.env_form_active?.remove_content_file(idx);
      }
    };

    // Replace specified position or append to the end
    const desired_position = index ?? Infinity;
    const position = Math.min(desired_position, lst.children.length);

    const old_child = this.file_lists[role].at(position);
    if (old_child) {
      lst.insertBefore(line_item, old_child.list_item);
      lst.removeChild(old_child.list_item);
    } else {
      lst.appendChild(line_item);
    }
    this.file_lists[role].splice(position, 1, file_record);
  }

  remove_from_file_list(role: Role, index: number) {
    const remove_button = this.file_lists[role][index].list_item.querySelector(
      'button[title="Remove"]',
    ) as HTMLButtonElement;
    remove_button.click();
  }

  /**
   * Prefill the add work form with information from an existing instance
   * @param instance_id The ID of the existing instance.
   */
  async update_work_instanceenv_info(instance_id: string) {
    const full_instance = (await (
      await fetch(
        `${this.base_url}/instances/${encodeURIComponent(instance_id)}`,
        { headers: [["Accept", "application/json"]] },
      )
    ).json()) as FullInstance;

    this.environment = full_instance.environment;
    const core_chooser = document.querySelector<HTMLSelectElement>(
      "#work-info select[name=work_core_chooser]",
    )!;
    core_chooser.selectedIndex = Array.from(core_chooser.options).findIndex(
      (o) =>
        o.dataset["coreName"] == this.environment.environment_core_name &&
        o.dataset["corePlatform"] == this.environment.environment_platform,
    );
    this.instance = full_instance.info;
    this.work = full_instance.work;

    const env_config_string = JSON.stringify(
      full_instance.environment.environment_config || {},
      null,
      2,
    );
    const env_configs = Array.from(
      document.querySelectorAll<HTMLTextAreaElement>(
        "textarea[name=env_config]",
      ),
    );
    for (const env_config of env_configs) {
      env_config.value = env_config_string;
    }

    // env_form_state_sync will sync the form's current config, but we want to use a new one later
    const saved_env_conf = this.environment.environment_config;
    this.clear_file_lists();
    this.env_form_state_sync();
    full_instance.objects.sort(
      (a: ObjectLink, b: ObjectLink) =>
        a.object_role_index - b.object_role_index,
    );
    for (const lnk of full_instance.objects) {
      this.add_to_file_list(lnk.object_role, lnk.file_filename, {
        existing: lnk.object_id,
      });
    }

    // Apply to env config form
    const content_file_map = this.file_lists.content.map(
      (file) => file.filename,
    );
    this.env_form_active?.set_env_conf(saved_env_conf ?? {}, content_file_map);
    this.environment.environment_config =
      this.env_form_active?.get_env_conf() ?? saved_env_conf;

    // TODO: fetch other bibliographic info fields...
  }

  showTab(
    targetTab: HTMLButtonElement,
    tabs: HTMLButtonElement[],
    tabPanels: HTMLDivElement[],
  ) {
    for (const tab of tabs) {
      // select/unselect tabs
      if (tab === targetTab) continue;
      tab.setAttribute("aria-selected", "false");
      tab.tabIndex = -1;
    }
    targetTab.setAttribute("aria-selected", "true");
    targetTab.tabIndex = 0;

    // hide/show tab panels
    const targetTabPanel = document.getElementById(
      targetTab.getAttribute("aria-controls")!,
    );
    for (const panel of tabPanels) {
      if (panel === targetTabPanel) continue;
      panel.hidden = true;
    }
    targetTabPanel!.hidden = false;
  }
}
