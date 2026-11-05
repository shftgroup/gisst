declare global {
  interface HTMLElementEventMap {
    "gisst-env-form-on-new-conf": GISSTEnvFormOnNewConfEvent;
    "gisst-env-form-on-content-add": GISSTEnvFormOnContentAddEvent;
    "gisst-env-form-on-content-remove": GISSTEnvFormOnContentRemoveEvent;
  }
}

export class GISSTEnvFormOnNewConfEvent extends Event {
  constructor(public readonly new_conf: object) {
    super("gisst-env-form-on-new-conf", {
      bubbles: true,
      composed: true,
    });
  }
}

export class GISSTEnvFormOnContentAddEvent extends Event {
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

export class GISSTEnvFormOnContentRemoveEvent extends Event {
  constructor(public readonly index: number) {
    super("gisst-env-form-on-content-remove", {
      bubbles: true,
      composed: true,
    });
  }
}

/**
 * This abstract class represents a form that can generate an environment
 * config for a specific emulation framework.
 *
 * Forms may declare that they handle content files. If so, users must not
 * be allowed to add or reorder them manually apart from through the form.
 *
 * Users should listen to the three relevant events for this form.
 * - {@link GISSTEnvFormOnNewConfEvent}
 * - {@link GISSTEnvFormOnContentAddEvent}
 * - {@link GISSTEnvFormOnContentRemoveEvent}
 */
export abstract class GISSTNewInstanceEnvForm extends HTMLElement {
  public abstract readonly HANDLES_CONTENT_FILES: boolean;

  /**
   * Retrieve the currently generated environment config.
   */
  abstract get_env_conf(): object;

  /**
   * Given an environment config, update the form to align with the given config.
   * The entire state of the form is reset to align with the given parameters.
   * @param new_conf The new environment config for the form.
   * @param content_file_map A map of `$CONTENT{X}` to file name strings. For each index X,
   *                         content_file_map[X] should be the corresponding file name.
   */
  abstract set_env_conf(new_conf: object, content_file_map: string[]): void;

  /**
   * Reset the form to its default state.
   */
  abstract reset(): void;

  /**
   * If a form manages content files and one is removed externally, notify the
   * form using this method.
   * @param index The index of the content file being removed
   */
  abstract remove_content_file(index: number): void;

  /**
   * Copies the text in the environment configuration textarea (aka the preview).
   * @param env_preview The textarea to copy from.
   * @param env_copy_button The button that triggered the action.
   */
  protected async copy_config(
    env_preview: HTMLTextAreaElement,
    env_copy_button: HTMLButtonElement,
  ) {
    await navigator.clipboard.writeText(env_preview.value);

    const original_text = env_copy_button.textContent;
    env_copy_button.textContent = "Copied!";
    env_copy_button.disabled = true;
    env_copy_button.classList.remove("gisst-Search-btn-primary");
    env_copy_button.classList.add("gisst-Search-btn-secondary");

    setTimeout(() => {
      env_copy_button.textContent = original_text;
      env_copy_button.disabled = false;
      env_copy_button.classList.remove("gisst-Search-btn-secondary");
      env_copy_button.classList.add("gisst-Search-btn-primary");
    }, 2000);
  }

  /**
   * Helper to extract the index (X) from a url string `$CONTENT{X}`.
   * @param url The string to extract the content file index from.
   * @returns The index X or null if not found.
   */
  protected extract_content_index(url: string): number | null {
    // Extract content number, i.e. "$CONTENT{X}"
    const parsed = url.match(/^\$CONTENT(\d*)$/m);
    const index = parsed?.[1];
    return index !== undefined ? Number(index) : null;
  }
}
