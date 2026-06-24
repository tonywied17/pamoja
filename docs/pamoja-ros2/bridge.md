# pamoja-ros2::bridge

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The live ROS 2 bridge: a ROS 2 node whose pub/sub are pamoja sensors and actuators.

[`Ros2Node`] wraps an `r2r` node and hands out typed publishers and subscribers exposed through
the core device model: a publisher is an [`Actuator`](pamoja_core::Actuator) whose command is a
ROS 2 message, and a subscriber is a [`Sensor`](pamoja_core::Sensor) whose reading is the next
message. A ROS 2 robot then drives like any other pamoja device, from any language binding.

A ROS 2 node only makes progress while it is spun, so create every publisher and subscriber on
the node, then drive [`spin_once`](Ros2Node::spin_once) in a loop (commonly on its own thread)
while the sensors and actuators are used from async tasks.

## struct `Ros2Node`

A ROS 2 node that produces pamoja sensors and actuators.

### `Ros2Node::new`

Creates a ROS 2 node.

**Arguments**

* `name` - the node name.
* `namespace` - the node namespace, or `""` for the default.

**Returns**

The node.

**Errors**

Returns [`Error::Transport`](pamoja_core::Error::Transport) if the ROS 2 context or node
cannot be created.

```rust
fn new(name: &str, namespace: &str) -> Result <Self>
```

### `Ros2Node::publisher`

Creates a publisher on a topic, exposed as an [`Actuator`].

**Arguments**

* `topic` - the topic to publish on.

**Returns**

A [`RosPublisher`] whose command type is the ROS 2 message `T`.

**Errors**

Returns [`Error::Transport`](pamoja_core::Error::Transport) if the publisher cannot be
created.

```rust
fn publisher <T: WrappedTypesupport>(&mut self, topic: &str) -> Result <RosPublisher <T>>
```

### `Ros2Node::subscriber`

Subscribes to a topic, exposed as a [`Sensor`].

**Arguments**

* `topic` - the topic to subscribe to.

**Returns**

A [`RosSubscriber`] whose reading type is the ROS 2 message `T`.

**Errors**

Returns [`Error::Transport`](pamoja_core::Error::Transport) if the subscription cannot be
created.

```rust
fn subscriber <T: WrappedTypesupport + Send + 'static>(&mut self, topic: &str,) -> Result <RosSubscriber <T>>
```

### `Ros2Node::service`

Offers a service on a name, exposed as a stream of requests to answer.

**Arguments**

* `name` - the service name.

**Returns**

A [`RosService`] whose requests and responses are the service type `S`.

**Errors**

Returns [`Error::Transport`](pamoja_core::Error::Transport) if the service cannot be created.

```rust
fn service <S: r2r::WrappedServiceTypeSupport + Send + 'static>(&mut self, name: &str,) -> Result <RosService <S>>
```

### `Ros2Node::client`

Creates a client for a service on a name.

**Arguments**

* `name` - the service name.

**Returns**

A [`RosClient`] for the service type `S`.

**Errors**

Returns [`Error::Transport`](pamoja_core::Error::Transport) if the client cannot be created.

```rust
fn client <S: r2r::WrappedServiceTypeSupport + 'static>(&mut self, name: &str,) -> Result <RosClient <S>>
```

### `Ros2Node::action_client`

Creates a client for an action on a name.

**Arguments**

* `name` - the action name.

**Returns**

A [`RosActionClient`] for the action type `T`.

**Errors**

Returns [`Error::Transport`](pamoja_core::Error::Transport) if the client cannot be created.

```rust
fn action_client <T: r2r::WrappedActionTypeSupport + 'static>(&mut self, name: &str,) -> Result <RosActionClient <T>>
```

### `Ros2Node::action_server`

Offers an action server on a name, exposed as a stream of incoming goals.

**Arguments**

* `name` - the action name.

**Returns**

A [`RosActionServer`] yielding goal requests for the action type `T`.

**Errors**

Returns [`Error::Transport`](pamoja_core::Error::Transport) if the server cannot be created.

```rust
fn action_server <T: r2r::WrappedActionTypeSupport + Send + 'static>(&mut self, name: &str,) -> Result <RosActionServer <T>>
```

### `Ros2Node::spin_once`

Spins the node once, processing ready ROS 2 work and feeding the subscriptions.

**Arguments**

* `timeout` - the longest to block waiting for work.

```rust
fn spin_once(&mut self, timeout: Duration)
```

## struct `RosPublisher`

A ROS 2 publisher exposed as an [`Actuator`] whose command is a ROS 2 message.

## struct `RosSubscriber`

A ROS 2 subscription exposed as a [`Sensor`] whose reading is the next ROS 2 message.

## struct `RosService`

A ROS 2 service server: a stream of requests, each answered with its `respond` method.

### `RosService <S>::next_request`

Awaits the next service request.

**Returns**

`Some(request)` to answer with its `respond` method, or `None` once the service has ended.

```rust
async fn next_request(&mut self) -> Option <r2r::ServiceRequest <S>>
```

## struct `RosClient`

A ROS 2 service client.

### `RosClient <S>::ready`

Waits until a server for this service is available.

**Returns**

`Ok(())` once a matching server has been discovered.

**Errors**

Returns [`Error::Transport`](pamoja_core::Error::Transport) if availability cannot be queried.

```rust
async fn ready(&self) -> Result <()>
```

### `RosClient <S>::call`

Calls the service and awaits its response.

**Arguments**

* `request` - the request message.

**Returns**

The response message.

**Errors**

Returns [`Error::Transport`](pamoja_core::Error::Transport) if the call fails.

```rust
async fn call(&self, request: &S::Request) -> Result <S::Response>
```

## struct `RosActionClient`

A ROS 2 action client: sends goals to a long-running task and awaits their results.

### `RosActionClient <T>::ready`

Waits until an action server for this action is available.

**Returns**

`Ok(())` once a matching server has been discovered.

**Errors**

Returns [`Error::Transport`](pamoja_core::Error::Transport) if availability cannot be queried.

```rust
async fn ready(&self) -> Result <()>
```

### `RosActionClient <T>::send_goal`

Sends a goal and returns a handle to its feedback stream and eventual result.

**Arguments**

* `goal` - the goal message.

**Returns**

A [`RosGoal`] tracking the accepted goal.

**Errors**

Returns [`Error::Transport`](pamoja_core::Error::Transport) if the goal cannot be sent or is
rejected by the server.

```rust
async fn send_goal(&self, goal: T::Goal) -> Result <RosGoal <T>> where T::Result: Send + 'static, T::Feedback: Send + 'static,
```

## struct `RosGoal`

A handle to an accepted action goal: its feedback stream and its eventual result.

### `RosGoal <T>::next_feedback`

Awaits the next feedback message from the server.

**Returns**

`Some(feedback)` for the next update, or `None` once feedback has ended.

```rust
async fn next_feedback(&mut self) -> Option <T::Feedback>
```

### `RosGoal <T>::result`

Awaits the goal's final result.

**Returns**

The result message.

**Errors**

Returns [`Error::Transport`](pamoja_core::Error::Transport) if the goal fails or is aborted.

```rust
async fn result(self) -> Result <T::Result>
```

## struct `RosActionServer`

A ROS 2 action server: a stream of incoming goals to accept and fulfil.

### `RosActionServer <T>::next_goal`

Awaits the next incoming goal request, to be accepted with its `accept` method.

**Returns**

`Some(request)` for the next goal, or `None` once the server has ended.

```rust
async fn next_goal(&mut self) -> Option <r2r::ActionServerGoalRequest <T>>
```

