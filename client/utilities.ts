/* Utility Functions */

/**
 * Convenient non-null assertion helper function allows asserting that something is not
 * null or undefined without having to write a JSDoc type cast that has to
 * explicitly know the non-null type (which is error prone).
 */
export function nonNull<T>(item: T) {
  if (item === null || item === undefined) throw "item is null or undefined";
  return item;
}

/**
 * Convenient button assertion helper function allows asserting that something is
 * an HTMLButtonElement
 */
export function asButton<T>(item: T) {
  if (item === null || item === undefined) throw "item is null or undefined";
  if (item instanceof HTMLButtonElement) return item;
  throw "item is not a button";
}

/**
 * Sets an html element's innerText
 */
export function setText(id: string, text: string) {
  nonNull(document.getElementById(id)).innerText = text;
}

/**
 * Sets an html element's attribute
 */
export function setAttribute(id: string, attribute: string, value: string) {
  document.getElementById(id)?.setAttribute(attribute, value);
}

/**
 * Sets an html element's style attribute
 */
export function setStyleAttribute(id: string | string[], value: string) {
  if (id instanceof Array) {
    id.forEach((i) => {
      setAttribute(i, "style", value);
    });
  } else {
    setAttribute(id, "style", value);
  }
}

/**
 * Removes all elements with the given class name.
 */
export function removeElements(className: string) {
  document.querySelectorAll(className).forEach((e) => e.remove());
}

/**
 * Removes an element with the given id.
 */
export function removeElement(id: string) {
  document.getElementById(id)?.remove();
}

/**
 *
 * Removes an element's style from its classList
 */
export function removeStyle(id: string, style: string) {
  document.getElementById(id)?.classList.remove(style);
}

/**
 * Removes the hidden classname from an element's classlist
 */
export function removeHidden(id: string) {
  removeStyle(id, "hidden");
}

/**
 * Appends an HTML string to the innerHTML of the given element
 */
export function appendElements(id: string, html: string) {
  nonNull(document.getElementById(id)).innerHTML = html;
}
