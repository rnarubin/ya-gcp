//! An API for interacting with the [Pub/Sub](https://cloud.google.com/pubsub) message service.
//!
//! Publishing and topic management is done through the [`PublisherClient`], while reading data and
//! subscription management is done through the [`SubscriberClient`].

use crate::{
    auth::grpc::{AuthGrpcService, OAuthTokenSource},
    grpc::StatusCodeSet,
};
use std::fmt::Display;

// alias Status as this module's error type
pub use ::tonic::Status as Error;

pub use client_builder::{BuildError, MakeConnection, PubSubConfig, Uri};
pub use publish_sink::{PublishError, PublishTopicSink, SinkError};
pub use streaming_subscription::{
    AcknowledgeError, AcknowledgeToken, ModifyAcknowledgeError, StreamSubscription,
    StreamSubscriptionConfig,
};

pub(crate) mod client_builder;
mod publish_sink;
mod streaming_subscription;

#[cfg(feature = "emulators")]
#[cfg_attr(docsrs, doc(cfg(feature = "emulators")))]
pub mod emulator;

/// Types and functions generated from PubSub's gRPC schema
#[allow(rustdoc::broken_intra_doc_links, rustdoc::bare_urls, missing_docs)]
pub mod api {
    include!("../generated/google.pubsub.v1.rs");

    // re-exports of prost types used within the generated code for convenience
    pub use prost_types::{Duration, FieldMask, Timestamp};
}

/// A set of status codes which should generally be retried when returned from an unsuccessful
/// operation.
///
/// See notes on retry recommendations in the pubsub docs
/// <https://cloud.google.com/pubsub/docs/reference/error-codes>
pub const DEFAULT_RETRY_CODES: StatusCodeSet = StatusCodeSet::new(&[
    tonic::Code::DeadlineExceeded,
    tonic::Code::Internal,
    tonic::Code::ResourceExhausted,
    tonic::Code::Unavailable,
]);

/// A client through which pubsub messages are sent, and topics are managed. Created
/// from the [`build_pubsub_publisher`](crate::builder::ClientBuilder::build_pubsub_publisher)
/// function.
///
/// This builds on top of the raw [gRPC publisher API](api::publisher_client::PublisherClient)
/// to provide more ergonomic functionality
#[derive(Debug, Clone)]
pub struct PublisherClient<C = crate::DefaultConnector> {
    inner: api::publisher_client::PublisherClient<
        AuthGrpcService<tonic::transport::Channel, OAuthTokenSource<C>>,
    >,
}

impl<C> PublisherClient<C>
where
    C: crate::Connect + Clone + Send + Sync + 'static,
{
    /// Create a sink which will publish [messages](api::PubsubMessage) to the given topic.
    ///
    /// See the type's [documentation](PublishTopicSink) for more details.
    pub fn publish_topic_sink(&mut self, topic: ProjectTopicName) -> PublishTopicSink<C> {
        PublishTopicSink::new(self.inner.clone(), topic)
    }
}

impl<C> std::ops::Deref for PublisherClient<C> {
    type Target = api::publisher_client::PublisherClient<
        AuthGrpcService<tonic::transport::Channel, OAuthTokenSource<C>>,
    >;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<C> std::ops::DerefMut for PublisherClient<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// A client through which pubsub messages are consumed, and subscriptions are managed. Created
/// from the [`build_pubsub_subscriber`](crate::builder::ClientBuilder::build_pubsub_subscriber)
/// function.
///
/// This is an interface built on top of the raw [gRPC subscriber
/// API](api::subscriber_client::SubscriberClient) which provides more ergonomic functionality
#[derive(Debug, Clone)]
pub struct SubscriberClient<C = crate::DefaultConnector> {
    inner: api::subscriber_client::SubscriberClient<
        AuthGrpcService<tonic::transport::Channel, OAuthTokenSource<C>>,
    >,
}

impl<C> SubscriberClient<C>
where
    C: crate::Connect + Clone + Send + Sync + 'static,
{
    /// Start a streaming subscription with the pubsub service.
    ///
    /// The returned stream will yield [messages](api::PubsubMessage) along with corresponding
    /// [`AcknowledgeTokens`](AcknowledgeToken), used to control message re-delivery.
    pub fn stream_subscription(
        &mut self,
        subscription: ProjectSubscriptionName,
        config: StreamSubscriptionConfig,
    ) -> StreamSubscription<C> {
        StreamSubscription::new(self.inner.clone(), subscription.into(), config)
    }
}

impl<C> std::ops::Deref for SubscriberClient<C> {
    type Target = api::subscriber_client::SubscriberClient<
        AuthGrpcService<tonic::transport::Channel, OAuthTokenSource<C>>,
    >;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<C> std::ops::DerefMut for SubscriberClient<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// A project and subscription name combined in a format expected by API calls,
///
/// ```
/// use ya_gcp::pubsub::ProjectSubscriptionName;
///
/// assert_eq!(
///     String::from(ProjectSubscriptionName::new(
///         "my-project",
///         "my-subscription"
///     )),
///     "projects/my-project/subscriptions/my-subscription".to_string(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectSubscriptionName(String);

impl ProjectSubscriptionName {
    /// Create a new `ProjectSubscriptionName` from the given project and subscription names
    pub fn new(project_name: impl Display, subscription_name: impl Display) -> Self {
        Self(format!(
            "projects/{project}/subscriptions/{subscription}",
            project = project_name,
            subscription = subscription_name
        ))
    }
}

impl From<ProjectSubscriptionName> for String {
    fn from(from: ProjectSubscriptionName) -> String {
        from.0
    }
}

impl std::fmt::Display for ProjectSubscriptionName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

/// A project and topic name combined in a format expected by API calls
///
/// ```
/// use ya_gcp::pubsub::ProjectTopicName;
///
/// assert_eq!(
///     String::from(ProjectTopicName::new("my-project", "my-topic")),
///     "projects/my-project/topics/my-topic".to_string(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectTopicName(String);

impl ProjectTopicName {
    /// Create a new `ProjectTopicName` from the given project and topic names
    pub fn new(project_name: impl Display, topic_name: impl Display) -> Self {
        Self(format!(
            "projects/{project}/topics/{topic}",
            project = project_name,
            topic = topic_name,
        ))
    }
}

impl From<ProjectTopicName> for String {
    fn from(from: ProjectTopicName) -> String {
        from.0
    }
}

impl std::fmt::Display for ProjectTopicName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
