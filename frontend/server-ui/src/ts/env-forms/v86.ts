import html from "../../html/env_form_v86.html?raw";
import styles from "../../css/server-ui-all.css?inline";
import { V86ImageType, V86Options } from "../v86";

const V86Images = ["hda", "hdb", "fda", "fdb", "cdrom"] as const;

// Initialise styles and template
const STYLE_SHEET = new CSSStyleSheet();
STYLE_SHEET.replaceSync(styles);
const TEMPLATE = document.createElement("template");
TEMPLATE.innerHTML = html;

declare global {
  interface HTMLElementEventMap {
    "gisst-env-form-on-new-conf": GISSTEnvFormOnNewConfEvent;
    "gisst-env-form-on-content-add": GISSTEnvFormOnContentAddEvent;
    "gisst-env-form-on-content-remove": GISSTEnvFormOnContentRemoveEvent;
  }
}

class GISSTEnvFormOnNewConfEvent extends Event {
  constructor(public readonly new_conf: object) {
    super("gisst-env-form-on-new-conf", {
      bubbles: true,
      composed: true,
    });
  }
}

class GISSTEnvFormOnContentAddEvent extends Event {
  constructor(
    public readonly content_file: File,
    public readonly index: number,
  ) {
    super("gisst-env-form-on-content-add", {
      bubbles: true,
      composed: true,
    });
  }
}

class GISSTEnvFormOnContentRemoveEvent extends Event {
  constructor(public readonly index: number) {
    super("gisst-env-form-on-content-remove", {
      bubbles: true,
      composed: true,
    });
  }
}

/**
 * Environment configuration form for V86.
 */
export class GISSTNewInstanceEnvFormV86
  extends HTMLElement
  implements GISSTNewInstanceEnvForm
{
  public readonly HANDLES_CONTENT_FILES = true;
  private env_conf: V86Options;
  private cur_content_idx: number;

  private shadow: ShadowRoot;
  private form: HTMLFormElement;
  private img_uploads: HTMLInputElement[];
  private img_upload_labels: HTMLParagraphElement[];
  private mem_input: HTMLInputElement;
  private gpu_mem_input: HTMLInputElement;
  private bios_chooser: HTMLInputElement;
  private vga_bios_chooser: HTMLInputElement;
  private env_preview: HTMLTextAreaElement;

  constructor() {
    super();
    this.env_conf = {};
    this.cur_content_idx = 0;
    this.shadow = this.attachShadow({ mode: "open" });

    this.shadow.adoptedStyleSheets = [STYLE_SHEET];
    const fragment = document.importNode(TEMPLATE.content, true);

    // Find elements
    this.form = fragment.querySelector(
      "form.gisst-New-instance-config-form",
    ) as HTMLFormElement;
    this.img_uploads = Array.from(
      fragment.querySelectorAll('input[type="file"]'),
    );
    this.img_upload_labels = Array.from(
      fragment.querySelectorAll('input[type="file"] ~ p[data-image]'),
    );
    this.mem_input = fragment.querySelector(
      "#memory-size-select",
    ) as HTMLInputElement;
    this.gpu_mem_input = fragment.querySelector(
      "#gpu-memory-select",
    ) as HTMLInputElement;
    this.bios_chooser = fragment.querySelector(
      "#bios-chooser",
    ) as HTMLInputElement;
    this.vga_bios_chooser = fragment.querySelector(
      "#vga-bios-chooser",
    ) as HTMLInputElement;
    this.env_preview = fragment.querySelector(
      'textarea[name="env_config"]',
    ) as HTMLTextAreaElement;

    this.form.addEventListener("input", () => this.#generate_config());
    for (const img_upload of this.img_uploads) {
      img_upload.addEventListener(
        "input",
        () => (img_upload.dataset.changed = "true"),
      );
    }

    this.shadow.appendChild(fragment);
  }

  set_env_conf(new_conf: V86Options, content_file_map: string[]) {
    this.form.reset();
    this.env_conf = new_conf;
    this.cur_content_idx = content_file_map.length;

    for (const img_upload of this.img_uploads) {
      // Map the image types and content files together
      delete img_upload.dataset.existing_filename;
      const img_type = img_upload.name as V86ImageType;
      const img_url = new_conf[img_type]?.url;
      if (!img_url) continue;

      const img_idx = this.#extract_content_index(img_url);
      if (img_idx !== null) {
        img_upload.dataset.existing_filename = content_file_map[img_idx];
      }
    }

    // Update the form based on the new environment config
    this.#update_form();
  }

  reset() {
    this.set_env_conf({}, []);
  }

  remove_content_file(index: number) {
    for (const img_upload of this.img_uploads) {
      const img_type = img_upload.name as V86ImageType;
      const img_url = this.env_conf[img_type]?.url;
      if (!img_url) continue;

      if (this.#extract_content_index(img_url) === index) {
        delete img_upload.dataset.existing_filename;
        img_upload.value = "";
        --this.cur_content_idx;
        delete this.env_conf[img_type];

        // Correct indicies on remaining images
        for (const image_type of V86Images) {
          const image = this.env_conf[image_type];
          const url = image?.url;

          if (url) {
            const this_index = this.#extract_content_index(url);
            if (this_index !== null && this_index > index) {
              image.url = `$CONTENT${this_index - 1}`;
            }
          }
        }

        break;
      }
    }

    this.#generate_config();
  }

  /**
   * Resynchronizes the form based on the existing environment config.
   * Only needs to be run if a new one is set externally.
   */
  #update_form() {
    // Populate memory sizes
    this.mem_input.value =
      this.env_conf.memory_size !== undefined
        ? String(Math.round(this.env_conf.memory_size / 1048576))
        : "";
    this.gpu_mem_input.value =
      this.env_conf.vga_memory_size !== undefined
        ? String(Math.round(this.env_conf.vga_memory_size / 1048576))
        : "";

    // Populate BIOS
    this.bios_chooser.value = "bios-seabios";
    if (this.env_conf.bios?.url === "bochs-bios.bin") {
      this.bios_chooser.value = "bios-bochs";
    }
    this.vga_bios_chooser.value = "vga-bios-standard";
    if (this.env_conf.bios?.url === "bochs-vgabios.bin") {
      this.vga_bios_chooser.value = "vga-bios-bochs";
    }

    // Update file selects
    this.#update_img_names();

    // Update preview
    this.env_preview.value = JSON.stringify(this.env_conf, null, 2);
  }

  /**
   * Queries the current config and updates all the file name labels.
   */
  #update_img_names() {
    for (let i = 0; i < this.img_uploads.length; ++i) {
      const img_upload = this.img_uploads[i];
      const img_upload_label = this.img_upload_labels[i];
      img_upload_label.textContent =
        img_upload.files?.[0]?.name ??
        img_upload.dataset.existing_filename ??
        "No file chosen";
    }
  }

  /**
   * Generates a new environment configuration based on the filled out form.
   */
  #generate_config() {
    let old_config = this.env_conf;
    const form_data = new FormData(this.form);

    const memory_size_mib = Number(form_data.get("mem-size"));
    const vga_memory_size_mib = Number(form_data.get("gpu-memory"));

    const bios = form_data.get("bios") as "bios-seabios" | "bios-bochs";
    const vga_bios = form_data.get("vga-bios") as
      | "vga-bios-standard"
      | "vga-bios-bochs";

    // Build env config from form data
    let config: V86Options = {
      memory_size: memory_size_mib * 1048576,
      vga_memory_size: vga_memory_size_mib * 1048576,
      bios: {
        url: bios === "bios-seabios" ? "seabios.bin" : "bochs-bios.bin",
      },
      vga_bios: {
        url:
          vga_bios === "vga-bios-standard"
            ? "vgabios.bin"
            : "bochs-vgabios.bin",
      },
    };

    // Setup file <-> `$CONTENT{X}` mapping
    for (const img_upload of this.img_uploads) {
      const image_type = img_upload.name as V86ImageType;
      const old_image = old_config[image_type];

      if (img_upload.dataset.changed !== "true") {
        // No change so keep the original one
        config[image_type] = old_image;
        continue;
      }

      // Image is either a new selection or modification
      delete img_upload.dataset.changed;
      delete img_upload.dataset.existing_filename;
      const index = old_image
        ? this.#extract_content_index(old_image.url)!
        : this.cur_content_idx++;
      config[image_type] = {
        url: `$CONTENT${index}`,
        async: true,
      };
      this.dispatchEvent(
        new GISSTEnvFormOnContentAddEvent(img_upload.files![0], index),
      );
    }

    this.env_preview.value = JSON.stringify(config, null, 2);
    this.env_conf = config;
    this.#update_img_names();
    this.dispatchEvent(new GISSTEnvFormOnNewConfEvent(this.env_conf));
  }

  /**
   * Helper to extract the index (X) from a url string `$CONTENT{X}`.
   * @param url The string to extract the content file index from.
   * @returns The index X or null if not found.
   */
  #extract_content_index(url: string): number | null {
    // Extract content number, i.e. "$CONTENT{X}"
    const parsed = url.match(/^\$CONTENT(\d*)$/m);
    const index = parsed?.[1];
    return index !== undefined ? Number(index) : null;
  }
}

customElements.define(
  "gisst-new-instance-env-form-v86",
  GISSTNewInstanceEnvFormV86,
);
