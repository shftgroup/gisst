declare global {
  interface GisstEvents {
    "gisst-instances": CustomEvent;
    "gisst-add-work": CustomEvent;
    "gisst-mute": CustomEvent;
    "gisst-zoom": CustomEvent<string>;
    "gisst-save-state": CustomEvent;
    "gisst-start-performance": CustomEvent;
    "gisst-finish-performance": CustomEvent;
    "gisst-load-state": CustomEvent;
    "gisst-activate-save": CustomEvent;
    "gisst-play-replay": CustomEvent;
    "gisst-upload": CustomEvent;
    "gisst-download": CustomEvent;
    "gisst-edit": CustomEvent;
    "gisst-tab": CustomEvent<{ tabId: string }>;
  }

  class GisstElement extends HTMLElement {
    public addEventListener<T extends keyof GisstEvents>(
      type: T,
      listener: (this: GisstElement, ev: GisstEvents[T]) => void,
      options?: boolean | AddEventListenerOptions,
    ): void;
    public addEventListener(
      type: string,
      listener: (this: GisstElement, ev: Event) => void,
      options?: boolean | AddEventListenerOptions,
    ): void;
  }

  class EditorLayout extends GisstElement {
    connectedCallback(): void;
  }

  class EditorSidebar extends GisstElement {
    connectedCallback(): void;
  }

  class EditorTabs extends GisstElement {
    connectedCallback(): void;
  }

  class EditorTimeline extends GisstElement {
    connectedCallback(): void;
    get filter(): string;
    set filter(value: string);
    get selected(): string;
    set selected(value: string);
  }

  class EditorStage extends GisstElement {
    connectedCallback(): void;
    get mode(): string;
    set mode(value: string);
  }

  class EditorControls extends GisstElement {
    connectedCallback(): void;
    get muted(): boolean;
    set muted(value: boolean);
    get zoom(): string;
    set zoom(value: string);
  }

  class EditorStateCard extends GisstElement {
    connectedCallback(): void;
  }

  class EditorSaveCard extends GisstElement {
    connectedCallback(): void;
  }

  class EditorReplayCard extends GisstElement {
    connectedCallback(): void;
  }

  interface HTMLElementTagNameMap {
    "editor-layout": EditorLayout;
    "editor-sidebar": EditorSidebar;
    "editor-tabs": EditorTabs;
    "editor-timeline": EditorTimeline;
    "editor-stage": EditorStage;
    "editor-controls": EditorControls;
    "editor-state-card": EditorStateCard;
    "editor-save-card": EditorSaveCard;
    "editor-replay-card": EditorReplayCard;
  }
}

export {};
