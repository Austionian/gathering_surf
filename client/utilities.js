/* Utility Functions */

/**
 * Convenient non-null assertion helper function allows asserting that something is not
 * null or undefined without having to write a JSDoc type cast that has to
 * explicitly know the non-null type (which is error prone).
 *
 * @template {any} T
 * @param {T} item
 */
export function nonNull(item) {
  if (item === null || item === undefined) throw "item is null or undefined";
  return item;
}

/**
 * Convenient button assertion helper function allows asserting that something is
 * an HTMLButtonElement
 *
 * @template {any} T
 * @param {T} item
 */
export function asButton(item) {
  if (item === null || item === undefined) throw "item is null or undefined";
  if (item instanceof HTMLButtonElement) return item;
  throw "item is not a button";
}

/**
 * Sets an html element's innerText
 *
 * @param {string} id
 * @param {string} text
 */
export function setText(id, text) {
  nonNull(document.getElementById(id)).innerText = text;
}

/**
 * Sets an html element's attribute
 *
 * @param {string} id
 * @param {string} attribute
 * @param {string} value
 */
export function setAttribute(id, attribute, value) {
  document.getElementById(id)?.setAttribute(attribute, value);
}

/**
 * Sets an html element's style attribute
 *
 * @param {string | string[]} id
 * @param {string} value
 */
export function setStyleAttribute(id, value) {
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
 *
 * @param {string} className
 */
export function removeElements(className) {
  document.querySelectorAll(className).forEach((e) => e.remove());
}

/**
 * Removes an element with the given id.
 *
 * @param {string} id
 */
export function removeElement(id) {
  document.getElementById(id)?.remove();
}

/**
 *
 * Removes an element's style from its classList
 *
 * @param {string} id
 * @param {string} style
 */
export function removeStyle(id, style) {
  document.getElementById(id)?.classList.remove(style);
}

/**
 * Removes the hidden classname from an element's classlist
 *
 * @param {string} id
 */
export function removeHidden(id) {
  removeStyle(id, "hidden");
}
