import html from "../../html/env_form_ra.html?raw";
import styles from "../../css/server-ui-all.css?inline";
import { GISSTNewInstanceEnvForm } from "./common";

// Initialise styles and template
const STYLE_SHEET = new CSSStyleSheet();
STYLE_SHEET.replaceSync(styles);
const TEMPLATE = document.createElement("template");
TEMPLATE.innerHTML = html;

/**
 * Environment configuration form for RA.
 */
export class GISSTNewInstanceEnvFormRA extends GISSTNewInstanceEnvForm {
  public readonly HANDLES_CONTENT_FILES = true;

  private shadow: ShadowRoot;
  private env_copy_button: HTMLButtonElement;
  private env_preview: HTMLTextAreaElement;

  constructor() {
    super();
    this.shadow = this.attachShadow({ mode: "open" });

    this.shadow.adoptedStyleSheets = [STYLE_SHEET];
    const fragment = document.importNode(TEMPLATE.content, true);

    // Find elements
    this.env_copy_button = fragment.querySelector(
      "#env-config-form-copy-btn",
    ) as HTMLButtonElement;
    this.env_preview = fragment.querySelector(
      'textarea[name="env_config"]',
    ) as HTMLTextAreaElement;

    this.env_copy_button.addEventListener(
      "click",
      () => void this.copy_config(this.env_preview, this.env_copy_button),
    );

    this.shadow.appendChild(fragment);
  }

  get_env_conf(): object {
    return {};
  }

  set_env_conf(_new_conf: object, _content_file_map: string[]) {}

  reset() {
    this.set_env_conf({}, []);
  }

  remove_content_file(_index: number) {}
}

customElements.define(
  "gisst-new-instance-env-form-ra",
  GISSTNewInstanceEnvFormRA,
);
