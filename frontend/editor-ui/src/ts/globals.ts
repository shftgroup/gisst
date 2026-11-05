class GisstElement extends HTMLElement {
  // Forwarding to HTMLElement.addEventListener. The event stays a plain Event -- clarity's sake.
  // GisstEvents is declared on the separate GisstElement in typing.d.ts.
  public addEventListener(
    type: string,
    listener: (this: GisstElement, ev: Event) => void,
    options?: boolean | AddEventListenerOptions,
  ): void {
    super.addEventListener(type, listener, options);
  }
}

export { GisstElement };
