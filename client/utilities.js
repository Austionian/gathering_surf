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

/**
 * Appends an HTML string to the innerHTML of the given element
 *
 * @param {string} id
 * @param {string} html
 */
export function appendElements(id, html) {
  nonNull(document.getElementById(id)).innerHTML = html;
}

/**
 * Appends the caution svg to an element's innerHTML
 *
 * @param {string} id
 */
function appendCaution(id) {
  document.getElementById(id).innerHTML +=
    `<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true" data-slot="icon">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z" />
              </svg>`;
}

/**
 * Updates the as of container to show it is out of date
 *
 * @param {string | string[]} id
 */
export function outOfDate(id) {
  setStyleAttribute(
    id,
    "background-color: #facc15; color: #000; display: flex; align-items: center; flex-direction: row-reverse; gap: 8px",
  );

  if (id instanceof Array) {
    id.forEach((i) => {
      appendCaution(i);
    });
  } else {
    appendCaution(id);
  }
}
