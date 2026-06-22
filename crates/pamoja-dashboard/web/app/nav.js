// nav.js - overlay navigation via zQuery router substates.
//
// Every overlay (modals, drawers, the network view and its docked panels) opens with
// open() and closes with back(). Opening pushes a sub-route history entry; the browser
// Back button and Escape both unwind the stack in LIFO order, closing the topmost
// overlay first. History is the single source of truth, so a sensor opened from the
// network backs out to the network, and one opened from the grid backs out to the grid.
//
// zQuery fires onSubstate on a back-pop with action 'pop' (when another substate is now
// current) or 'reset'/null (when the last substate was left). We close the topmost
// overlay on ANY such pop and return true to consume it, so the route is never
// re-resolved underneath the overlay. See zQuery docs: Router > Sub-Route Substates.

let router;
const stack = [];

/** Wires the substate listener. Call once with the router after it is created. */
export function initNav(r)
{
  router = r || ($.getRouter && $.getRouter());
  if (!router) return;
  router.onSubstate(() =>
  {
    if (!stack.length) return false; // no overlay open: let normal route navigation happen
    const close = stack.pop();
    if (close) close();
    return true; // consume: do not re-resolve the route
  });
}

/** Opens an overlay: runs `openFn`, records `closeFn`, and pushes a history entry. */
export function open(openFn, closeFn)
{
  openFn();
  stack.push(closeFn);
  if (router) router.pushSubstate('ov');
}

/** Closes the topmost overlay by going back one history entry. */
export function back()
{
  if (stack.length) history.back();
}
