/**
 * An interface representing a form that can generate an environment config
 * for a specific emulation framework.
 *
 * Forms may declare that they handle content files. If so, users must not
 * be allowed to add or reorder them manually apart from through the form.
 *
 * Users should listen to the three relevant events for this form.
 * - {@link GISSTEnvFormOnNewConfEvent}
 * - {@link GISSTEnvFormOnContentAddEvent}
 * - {@link GISSTEnvFormOnContentRemoveEvent}
 */
interface GISSTNewInstanceEnvForm {
  readonly HANDLES_CONTENT_FILES: boolean;

  /**
   * Given an environment config, update the form to align with the given config.
   * The entire state of the form is reset to align with the given parameters.
   * @param new_conf The new environment config for the form.
   * @param content_file_map A map of `$CONTENT{X}` to file name strings. For each index X,
   *                         content_file_map[X] should be the corresponding file name.
   */
  set_env_conf(new_conf: object, content_file_map: string[]): void;

  /**
   * Reset the form to its default state.
   */
  reset(): void;

  /**
   * Notifies the form of the removal of a content file at a specific index.
   * @param index The index of the content file being removed
   */
  remove_content_file(index: number): void;
}
