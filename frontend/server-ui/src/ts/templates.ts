// Import all templates from templates.html
import templates from "../html/templates.html?raw";
const TEMPLATES = document.createElement("template");
TEMPLATES.innerHTML = templates.trim();

/**
 * Create a new instance of an element from a template.
 * @param template_id ID of the template to instantiate.
 * @returns a {@link DocumentFragment} of the template being imported.
 */
export function elementFromTemplate<T extends Element>(template_id: string): T {
  const template_element = TEMPLATES.content.querySelector(
    "#" + template_id,
  ) as HTMLTemplateElement;
  const content = document.importNode(
    template_element.content,
    true,
  ).firstElementChild;
  if (!content) {
    throw new Error("Element not found or wrong type", {
      cause: { template_id, content },
    });
  }
  return content as T;
}
