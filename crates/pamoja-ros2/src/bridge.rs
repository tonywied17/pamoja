//! The live ROS 2 bridge: a ROS 2 node whose pub/sub are pamoja sensors and actuators.
//!
//! [`Ros2Node`] wraps an `r2r` node and hands out typed publishers and subscribers exposed through
//! the core device model: a publisher is an [`Actuator`](pamoja_core::Actuator) whose command is a
//! ROS 2 message, and a subscriber is a [`Sensor`](pamoja_core::Sensor) whose reading is the next
//! message. A ROS 2 robot then drives like any other pamoja device, from any language binding.
//!
//! A ROS 2 node only makes progress while it is spun, so create every publisher and subscriber on
//! the node, then drive [`spin_once`](Ros2Node::spin_once) in a loop (commonly on its own thread)
//! while the sensors and actuators are used from async tasks.

use std::pin::Pin;
use std::time::Duration;

use futures::stream::{Stream, StreamExt};
use pamoja_core::{Actuator, Error, Result, Sensor};
use r2r::{Context, Node, Publisher, QosProfile, WrappedTypesupport};

/// A ROS 2 node that produces pamoja sensors and actuators.
pub struct Ros2Node {
    node: Node,
}

impl Ros2Node {
    /// Creates a ROS 2 node.
    ///
    /// # Arguments
    ///
    /// * `name` - the node name.
    /// * `namespace` - the node namespace, or `""` for the default.
    ///
    /// # Returns
    ///
    /// The node.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`](pamoja_core::Error::Transport) if the ROS 2 context or node
    /// cannot be created.
    pub fn new(name: &str, namespace: &str) -> Result<Self> {
        let context = Context::create().map_err(map_err)?;
        let node = Node::create(context, name, namespace).map_err(map_err)?;
        Ok(Self { node })
    }

    /// Creates a publisher on a topic, exposed as an [`Actuator`].
    ///
    /// # Arguments
    ///
    /// * `topic` - the topic to publish on.
    ///
    /// # Returns
    ///
    /// A [`RosPublisher`] whose command type is the ROS 2 message `T`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`](pamoja_core::Error::Transport) if the publisher cannot be
    /// created.
    pub fn publisher<T: WrappedTypesupport>(&mut self, topic: &str) -> Result<RosPublisher<T>> {
        let publisher = self
            .node
            .create_publisher::<T>(topic, QosProfile::default())
            .map_err(map_err)?;
        Ok(RosPublisher { publisher })
    }

    /// Subscribes to a topic, exposed as a [`Sensor`].
    ///
    /// # Arguments
    ///
    /// * `topic` - the topic to subscribe to.
    ///
    /// # Returns
    ///
    /// A [`RosSubscriber`] whose reading type is the ROS 2 message `T`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`](pamoja_core::Error::Transport) if the subscription cannot be
    /// created.
    pub fn subscriber<T: WrappedTypesupport + Send + 'static>(
        &mut self,
        topic: &str,
    ) -> Result<RosSubscriber<T>> {
        let stream = self
            .node
            .subscribe::<T>(topic, QosProfile::default())
            .map_err(map_err)?;
        Ok(RosSubscriber {
            stream: Box::pin(stream),
        })
    }

    /// Spins the node once, processing ready ROS 2 work and feeding the subscriptions.
    ///
    /// # Arguments
    ///
    /// * `timeout` - the longest to block waiting for work.
    pub fn spin_once(&mut self, timeout: Duration) {
        self.node.spin_once(timeout);
    }
}

/// A ROS 2 publisher exposed as an [`Actuator`] whose command is a ROS 2 message.
pub struct RosPublisher<T: WrappedTypesupport> {
    publisher: Publisher<T>,
}

impl<T: WrappedTypesupport + 'static> Actuator for RosPublisher<T> {
    type Command = T;

    async fn apply(&mut self, command: T) -> Result<()> {
        self.publisher.publish(&command).map_err(map_err)
    }
}

/// A ROS 2 subscription exposed as a [`Sensor`] whose reading is the next ROS 2 message.
pub struct RosSubscriber<T> {
    stream: Pin<Box<dyn Stream<Item = T> + Send>>,
}

impl<T> Sensor for RosSubscriber<T> {
    type Reading = T;

    async fn read(&mut self) -> Result<T> {
        self.stream.next().await.ok_or(Error::Closed)
    }
}

fn map_err<E: core::fmt::Display>(err: E) -> Error {
    Error::Transport(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn chatter_round_trips_through_ros2() {
        let mut node = Ros2Node::new("pamoja_bridge_test", "").unwrap();
        let mut publisher = node
            .publisher::<r2r::std_msgs::msg::String>("/pamoja_chatter")
            .unwrap();
        let mut subscriber = node
            .subscriber::<r2r::std_msgs::msg::String>("/pamoja_chatter")
            .unwrap();

        // Spin the node on a dedicated thread so the subscription stream is fed.
        let spinner = std::thread::spawn(move || {
            for _ in 0..400 {
                node.spin_once(Duration::from_millis(50));
            }
        });

        // Publish until the subscriber sees it; a volatile subscription drops messages sent
        // before discovery completes, so retry rather than race it.
        let received = tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                publisher
                    .apply(r2r::std_msgs::msg::String {
                        data: "hello".to_string(),
                    })
                    .await
                    .unwrap();
                tokio::select! {
                    msg = subscriber.read() => return msg.unwrap(),
                    _ = tokio::time::sleep(Duration::from_millis(200)) => {}
                }
            }
        })
        .await
        .expect("a message should arrive within the timeout");

        assert_eq!(received.data, "hello");
        let _ = spinner.join();
    }
}
