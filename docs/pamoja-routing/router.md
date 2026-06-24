# pamoja-routing::router

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The routing table and the per-packet forwarding decision.

## struct `Route`

A learned route to a destination: the neighbour to send through, and the cost.

### `Route::dst`

Returns the destination node this route reaches.

**Returns**

The destination address.

```rust
fn dst(&self) -> u32
```

### `Route::next_hop`

Returns the neighbour to send through to reach the destination.

**Returns**

The next-hop address.

```rust
fn next_hop(&self) -> u32
```

### `Route::cost`

Returns the cost of this route, in whatever metric the caller reports (hop count,
summed link cost, or another).

**Returns**

The route cost; lower is better.

```rust
fn cost(&self) -> u16
```

## enum `Forward`

What to do with a packet bound for a given destination.

- `Deliver` - The packet is for this node; hand it to the application.
- `Relay` - A route is known; unicast the packet to this next hop.
- `Flood` - No route is known; fall back to flooding the packet.

## struct `Router`

A fixed-size routing table for one node.

The table holds up to `N` routes, learned from the traffic the node hears. It keeps the
cheapest route it knows to each destination, and when full it gives up the most
expensive route to make room for a cheaper one, so its limited memory holds the routes
most worth keeping.

**Examples**

```
use pamoja_routing::{Forward, Router};

let mut router: Router<8> = Router::new(0x0A);
router.observe(0x0B, 0x0C, 3); // reach 0x0B via 0x0C, cost 3
assert_eq!(router.next_hop(0x0B), Some(0x0C));
assert_eq!(router.forward(0x0A), Forward::Deliver); // a packet for us
```

### `Router <N>::new`

Creates an empty router for the node at `me`.

**Arguments**

* `me` - this node's address.

**Returns**

A router holding no routes.

```rust
const fn new(me: u32) -> Self
```

### `Router <N>::address`

Returns this node's address.

**Returns**

The address the router was created with.

```rust
fn address(&self) -> u32
```

### `Router <N>::observe`

Learns the way to a node from a packet heard from it.

A packet that originated at `origin` and reached this node via the neighbour `via`
proves `via` is a way back to `origin` at the reported `cost`. The router adopts the
route if it is cheaper than what it knows, or if it refreshes the cost of the route
it is already using, and ignores a route to itself.

**Arguments**

* `origin` - the node the packet came from, the destination this route reaches.
* `via` - the neighbour the packet arrived through, the next hop for this route.
* `cost` - the cost the packet reports for reaching `origin` through `via`.

**Returns**

`true` if the table changed (a route was added, redirected, or recosted), `false`
if the observation taught it nothing new.

```rust
fn observe(&mut self, origin: u32, via: u32, cost: u16) -> bool
```

### `Router <N>::next_hop`

Returns the next hop to reach a destination, if a route is known.

**Arguments**

* `dst` - the destination to reach.

**Returns**

The next-hop address, or [`None`] if no route is known.

```rust
fn next_hop(&self, dst: u32) -> Option <u32>
```

### `Router <N>::cost`

Returns the cost of the known route to a destination, if any.

**Arguments**

* `dst` - the destination to reach.

**Returns**

The route cost, or [`None`] if no route is known.

```rust
fn cost(&self, dst: u32) -> Option <u16>
```

### `Router <N>::route`

Returns the known route to a destination, if any.

**Arguments**

* `dst` - the destination to reach.

**Returns**

The [`Route`], or [`None`] if no route is known.

```rust
fn route(&self, dst: u32) -> Option <Route>
```

### `Router <N>::forward`

Decides what to do with a packet bound for a destination.

**Arguments**

* `dst` - the packet's destination.

**Returns**

[`Forward::Deliver`] if the packet is for this node, [`Forward::Relay`] with the
next hop if a route is known, or [`Forward::Flood`] otherwise.

```rust
fn forward(&self, dst: u32) -> Forward
```

### `Router <N>::forget`

Forgets the route to a destination, if one is held.

**Arguments**

* `dst` - the destination whose route to drop.

```rust
fn forget(&mut self, dst: u32)
```

### `Router <N>::len`

Returns how many routes the table currently holds.

**Returns**

The number of routes.

```rust
fn len(&self) -> usize
```

### `Router <N>::is_empty`

Reports whether the table holds no routes.

**Returns**

`true` if no routes are held.

```rust
fn is_empty(&self) -> bool
```

