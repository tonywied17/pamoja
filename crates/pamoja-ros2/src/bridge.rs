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

    /// Offers a service on a name, exposed as a stream of requests to answer.
    ///
    /// # Arguments
    ///
    /// * `name` - the service name.
    ///
    /// # Returns
    ///
    /// A [`RosService`] whose requests and responses are the service type `S`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`](pamoja_core::Error::Transport) if the service cannot be created.
    pub fn service<S: r2r::WrappedServiceTypeSupport + Send + 'static>(
        &mut self,
        name: &str,
    ) -> Result<RosService<S>> {
        let requests = self
            .node
            .create_service::<S>(name, QosProfile::default())
            .map_err(map_err)?;
        Ok(RosService {
            requests: Box::pin(requests),
        })
    }

    /// Creates a client for a service on a name.
    ///
    /// # Arguments
    ///
    /// * `name` - the service name.
    ///
    /// # Returns
    ///
    /// A [`RosClient`] for the service type `S`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`](pamoja_core::Error::Transport) if the client cannot be created.
    pub fn client<S: r2r::WrappedServiceTypeSupport + 'static>(
        &mut self,
        name: &str,
    ) -> Result<RosClient<S>> {
        let client = self
            .node
            .create_client::<S>(name, QosProfile::default())
            .map_err(map_err)?;
        Ok(RosClient { client })
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

/// A ROS 2 service server: a stream of requests, each answered with its `respond` method.
pub struct RosService<S>
where
    S: r2r::WrappedServiceTypeSupport,
{
    requests: Pin<Box<dyn Stream<Item = r2r::ServiceRequest<S>> + Send>>,
}

impl<S: r2r::WrappedServiceTypeSupport + 'static> RosService<S> {
    /// Awaits the next service request.
    ///
    /// # Returns
    ///
    /// `Some(request)` to answer with its `respond` method, or `None` once the service has ended.
    pub async fn next_request(&mut self) -> Option<r2r::ServiceRequest<S>> {
        self.requests.next().await
    }
}

/// A ROS 2 service client.
pub struct RosClient<S>
where
    S: r2r::WrappedServiceTypeSupport,
{
    client: r2r::Client<S>,
}

impl<S: r2r::WrappedServiceTypeSupport + 'static> RosClient<S> {
    /// Waits until a server for this service is available.
    ///
    /// # Returns
    ///
    /// `Ok(())` once a matching server has been discovered.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`](pamoja_core::Error::Transport) if availability cannot be queried.
    pub async fn ready(&self) -> Result<()> {
        r2r::Node::is_available(&self.client)
            .map_err(map_err)?
            .await
            .map_err(map_err)
    }

    /// Calls the service and awaits its response.
    ///
    /// # Arguments
    ///
    /// * `request` - the request message.
    ///
    /// # Returns
    ///
    /// The response message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`](pamoja_core::Error::Transport) if the call fails.
    pub async fn call(&self, request: &S::Request) -> Result<S::Response> {
        self.client
            .request(request)
            .map_err(map_err)?
            .await
            .map_err(map_err)
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

    // The cross-interop proof: a real ROS 2 publication, carried over Zenoh by rmw_zenoh, is
    // received by a plain pamoja `ZenohTransport` and decoded by our own CDR, with the live key
    // matching the structure `pamoja-ros2` builds. Ignored by default because it needs
    // `RMW_IMPLEMENTATION=rmw_zenoh_cpp` and peer discovery; run it with `cargo xtask ros`.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "needs rmw_zenoh; run via `cargo xtask ros` or RMW_IMPLEMENTATION=rmw_zenoh_cpp"]
    async fn ros2_twist_is_received_over_zenoh() {
        use crate::msg::Twist;
        use pamoja_core::Transport;
        use pamoja_zenoh::{ZenohConfig, ZenohTransport};
        use std::time::Duration;

        // pamoja side: a Zenoh peer subscribing to the cmd_vel key in ROS domain 0.
        let mut zenoh = ZenohTransport::new(ZenohConfig::new().multicast_scouting(true));
        zenoh.connect().await.unwrap();
        zenoh.subscribe("0/cmd_vel/**").await.unwrap();

        // ROS 2 side: an r2r publisher that, under rmw_zenoh, puts the Twist onto a Zenoh key.
        let mut node = Ros2Node::new("pamoja_interop_test", "").unwrap();
        let mut publisher = node
            .publisher::<r2r::geometry_msgs::msg::Twist>("/cmd_vel")
            .unwrap();
        let spinner = std::thread::spawn(move || {
            for _ in 0..400 {
                node.spin_once(Duration::from_millis(50));
            }
        });

        let sample = tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                publisher.apply(twist(0.6, 0.4)).await.unwrap();
                tokio::select! {
                    msg = zenoh.recv() => return msg.unwrap().unwrap(),
                    _ = tokio::time::sleep(Duration::from_millis(250)) => {}
                }
            }
        })
        .await
        .expect("a ROS 2 publication should arrive over Zenoh");

        // The live key matches the rmw_zenoh structure pamoja-ros2 builds for this topic and type.
        assert!(
            sample
                .key
                .starts_with("0/cmd_vel/geometry_msgs::msg::dds_::Twist_/RIHS01_"),
            "unexpected key: {}",
            sample.key,
        );

        // The payload is CDR our own decoder reads back to the published values.
        let decoded = Twist::from_cdr(&sample.payload).expect("payload should be a CDR Twist");
        assert!((decoded.linear.x - 0.6).abs() < 1e-9);
        assert!((decoded.angular.z - 0.4).abs() < 1e-9);

        let _ = spinner.join();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn set_bool_service_round_trips() {
        use r2r::std_srvs::srv::SetBool;
        use std::time::Duration;

        let mut node = Ros2Node::new("pamoja_service_test", "").unwrap();
        let mut service = node
            .service::<SetBool::Service>("/pamoja_set_bool")
            .unwrap();
        let client = node.client::<SetBool::Service>("/pamoja_set_bool").unwrap();

        let spinner = std::thread::spawn(move || {
            for _ in 0..400 {
                node.spin_once(Duration::from_millis(50));
            }
        });

        // The server answers one request, echoing the flag and a message.
        let server = tokio::spawn(async move {
            if let Some(request) = service.next_request().await {
                let response = SetBool::Response {
                    success: request.message.data,
                    message: "ok".to_string(),
                };
                let _ = request.respond(response);
            }
        });

        tokio::time::timeout(Duration::from_secs(10), client.ready())
            .await
            .expect("the service should become available")
            .unwrap();
        let response = tokio::time::timeout(
            Duration::from_secs(10),
            client.call(&SetBool::Request { data: true }),
        )
        .await
        .expect("the call should return")
        .unwrap();

        assert!(response.success);
        assert_eq!(response.message, "ok");

        let _ = server.await;
        let _ = spinner.join();
    }

    fn twist(vx: f64, wz: f64) -> r2r::geometry_msgs::msg::Twist {
        r2r::geometry_msgs::msg::Twist {
            linear: r2r::geometry_msgs::msg::Vector3 {
                x: vx,
                y: 0.0,
                z: 0.0,
            },
            angular: r2r::geometry_msgs::msg::Vector3 {
                x: 0.0,
                y: 0.0,
                z: wz,
            },
        }
    }
}
