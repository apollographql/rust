/**
 * For lack of a "home of federation utilities", this function is copy/pasted
 * verbatim across the federation, gateway, and query-planner packages. Any changes
 * made here should be reflected in the other two locations as well.
 *
 * @param condition
 * @param message
 * @throws
 */
export function assert(condition: any, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}
