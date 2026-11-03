import SVG_CITE from "../media/cite.svg?url";
import SVG_PLAY from "../media/play.svg?url";
import SVG_RECORD from "../media/record.svg?url";
import SVG_CLONE from "../media/clone.svg?url";
import { type TemplateParams, type Hit } from "instantsearch.js";
type HTML = TemplateParams["html"];
type Components = TemplateParams["components"];
type Ret = ReturnType<HTML>;

export interface InstanceRecord extends Hit {
  instance_id: string;
  work_version: string;
  work_platform: string;
}
export interface CreatorRecord extends InstanceRecord {
  creator_id: string;
  hidden: boolean;
}

export function hit_classes(
  hit: CreatorRecord,
  show_creator_info: boolean,
  show_instance_info: boolean,
): string {
  return `gisst-Search-results-row gisst-Search-responsive-results-row
           ${!show_creator_info ? "gisst-Search-no-creator" : ""}
           ${hit.hidden ? "gisst-Search-item-hidden" : ""}
           ${!show_instance_info ? "gisst-Search-no-instance" : ""}`;
}

export function created_item_info(
  base_url: string,
  modeltype: string,
  id: string,
  hit: CreatorRecord,
  short_title_key: string,
  long_title_key: string,
  screenshot: string | null,
  html: HTML,
  components: Components,
): Ret {
  return html` ${
      screenshot
        ? html`<div class="gisst-Search-cell gisst-Search-screenshot-cell">
            <img
              class="gisst-Search-screenshot"
              src="data:image/png;base64,${screenshot}"
              alt="Screenshot from instance ${hit.work_name}"
            />
          </div>`
        : html``
    }
    <div class="gisst-Search-cell gisst-Search-${modeltype}-info">
      <div class="gisst-Search-name">
        <a href="${base_url}/play/${hit.instance_id}?${modeltype}=${id}"
          >${components.Highlight({ hit, attribute: short_title_key })}</a
        >
      </div>
      <div class="gisst-Search-description">
        ${components.Highlight({ hit, attribute: long_title_key })}
      </div>
    </div>`;
}
export function instance_info(
  base_url: string,
  hit: InstanceRecord,
  html: HTML,
  components: Components,
): Ret {
  return html`
    <div class="gisst-Search-cell gisst-Search-instance-info">
      <div class="gisst-Search-instance-name">
        <a href="${base_url}/instances/${hit.instance_id}"
          >${components.Highlight({ hit, attribute: "work_name" })}</a
        >
      </div>
      <div class="gisst-Search-instance-details">
        ${hit.work_version} • ${hit.work_platform}
      </div>
    </div>
  `;
}
export function instance_info_3col(
  base_url: string,
  hit: InstanceRecord,
  html: HTML,
  components: Components,
): Ret {
  return html`
    <div class="gisst-Search-cell gisst-Search-instance-work-name">
      <a href="${base_url}/instances/${hit.instance_id}"
        >${components.Highlight({ hit, attribute: "work_name" })}</a
      >
    </div>
    <div class="gisst-Search-cell gisst-Search-instance-platform-info">
      ${components.Highlight({ hit, attribute: "work_platform" })}
    </div>
    <div class="gisst-Search-cell gisst-Search-instance-version-info">
      ${hit.work_version}
    </div>
  `;
}
export function creator_info(
  base_url: string,
  hit: CreatorRecord,
  html: HTML,
  components: Components,
): Ret {
  return html`
    <div class="gisst-Search-cell gisst-Search-creator-info">
      <a href="${base_url}/creators/${hit.creator_id}"
        >${components.Highlight({ hit, attribute: "creator_full_name" })}</a
      >
    </div>
  `;
}
export function hidden_state(id: string, hidden: boolean, html: HTML): Ret {
  return html`
                <div class="gisst-Search-cell gisst-Search-hidden-info">
                <input name="hidden-${id}" class="hide-show-checkbox" type="checkbox" data-id="${id}" checked="${hidden}"></input>
                </div>
                `;
}
export function hideshow_action(id: string, hidden: boolean, html: HTML): Ret {
  return html`<label for="hidden-${id}" class="gisst-Search-checkbox-label">Hide?</label>
              <input name="hidden-${id}" class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only hide-show-checkbox" type="checkbox" data-id="${id}" checked="${hidden}"></input>
              `;
}
export function play_actions(
  base_url: string,
  instance: string,
  id: string,
  modeltype: string,
  can_clone: boolean,
  html: HTML,
): Ret {
  const url = `${base_url}/play/${instance}${modeltype != instance ? `?${modeltype}=${id}` : ""}`;
  const cite_url = url.replace("/play/", "/data/");
  const clone_url = `${base_url}/instances/${instance}/clone?${modeltype}=${id}`;
  const record_url =
    modeltype == "instance"
      ? `${url}?boot_into_record=true`
      : `${url}&boot_into_record=true`;
  return html`<a
      class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only"
      href="${url}"
      >Play</a
    >
    <a
      class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only"
      href="${url}"
      title="Play"
    >
      <img src="${SVG_PLAY}" width="24" height="24" />
    </a>
    ${
      modeltype != "performance"
        ? html`
            <a
              class="gisst-Search-btn gisst-Search-btn-secondary gisst-Search-btn-text-only"
              href="${record_url}"
              >Record</a
            >
            <a
              class="gisst-Search-btn gisst-Search-btn-secondary gisst-Search-btn-icon gisst-Search-btn-icon-only"
              href="${record_url}"
            >
              <img src="${SVG_RECORD}" width="24" height="24" />
            </a>
          `
        : html``
    }
    <a
      class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-text-only"
      href="${cite_url}"
      >Cite</a
    >
    <a
      class="gisst-Search-btn gisst-Search-btn-primary gisst-Search-btn-icon gisst-Search-btn-icon-only"
      href="${cite_url}"
      title="Cite"
    >
      <img src="${SVG_CITE}" width="24" height="24" />
    </a>
    ${
      can_clone
        ? html`
            <a
              class="gisst-Search-btn gisst-Search-btn-accent gisst-Search-btn-text-only"
              href="${clone_url}"
              >Clone</a
            >
            <a
              class="gisst-Search-btn gisst-Search-btn-accent gisst-Search-btn-icon gisst-Search-btn-icon-only"
              href="${clone_url}"
            >
              <img src="${SVG_CLONE}" width="24" height="24" />
            </a>
          `
        : ""
    } `;
}

export function hideshow_click_listener(
  base_url: string,
  modeltype: string,
): (evt: Event) => void {
  return (event) => {
    void (async () => {
      if (
        event.target &&
        (event.target as HTMLElement).classList.contains("hide-show-checkbox")
      ) {
        const checkbox = event.target as HTMLInputElement;
        const id = checkbox.dataset["id"];
        try {
          await (
            await fetch(`${base_url}/${modeltype}/${id}/hide`, {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ state: checkbox.checked }),
            })
          ).text();
          if (checkbox.checked) {
            checkbox.parentElement?.parentElement?.classList.add(
              "gisst-Search-item-hidden",
            );
          } else {
            checkbox.parentElement?.parentElement?.classList.remove(
              "gisst-Search-item-hidden",
            );
          }
        } catch (error) {
          console.error(error);
          checkbox.checked = !checkbox.checked;
        }
      }
    })();
  };
}
