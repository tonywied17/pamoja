# command

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The control contract: the authenticated actions a client can ask a node to take.

Reading the dashboard needs no command; changing the node does. A [`Command`] arrives
over the authenticated `POST /command` path (see the serving layer) and is dispatched
to the [`StateSource`](crate::StateSource), which is the only thing that can move an
actuator or change the fleet. The wire form is a serde-tagged object, so the page
sends `{"type":"actuate", ...}`.

## enum `Command`

A control action a client asks the node to take.

The provisioning variants carry the group or sensor the client built, so the device
records the structure the operator described; the device owns and shares it across
every client.

- `Actuate` - Set a discrete actuator to one of its actions, such as opening a valve.
- `AddGroup` - Add a group to an organization.
- `RemoveGroup` - Remove a group by id.
- `AddSensor` - Add a sensor to a group.
- `RemoveSensor` - Remove a sensor by its `"groupId/sensorId"` path.

## enum `CommandError`

Why a command could not be carried out. The [`code`](CommandError::code) is a stable,
language-neutral string the page localizes.

- `Unsupported` - The source does not handle this kind of command.
- `UnknownTarget` - No actuator or target matches the command.
- `InvalidAction` - The target exists but does not accept the requested action.

### `CommandError::code`

Returns the stable error code for this failure.

**Returns**

A dotted, language-neutral code such as `"command.unknown_target"`.

```rust
fn code(self) -> &'static str
```

