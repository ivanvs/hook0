use chrono::{DateTime, Utc};
use clap::crate_version;
use hook0_client::Hook0Client;
use log::{info, warn};
use reqwest::Url;
use serde::Serialize;
use serde_json::{to_value, to_vec, Value};
use uuid::Uuid;

pub fn initialize(
    api_url: Option<Url>,
    application_id: Option<Uuid>,
    application_secret: Option<Uuid>,
) -> Option<Hook0Client> {
    match (api_url, application_id, application_secret) {
        (Some(url), Some(id), Some(secret)) => match Hook0Client::new(url, id, &secret) {
            Ok(client) => {
                info!(
                    "Events from this Hook0 instance will be sent to {} [application ID = {}]",
                    client.api_url(),
                    client.application_id()
                );
                Some(client)
            }
            Err(_e) => {
                warn!("Could not initialize a Hook0 client that will receive events from this Hook0 instance");
                None
            }
        },
        _ => {
            info!("No Hook0 client was configured to receive events from this Hook0 instance");
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum Hook0ClientEvent {
    OrganizationCreated(EventOrganizationCreated),
    EventTypeCreated(EventEventTypeCreated),
}

impl Hook0ClientEvent {
    pub fn mk_hook0_event<'a>(self) -> hook0_client::Event<'a> {
        fn to_event<'a, E: 'a + Event>(
            event: E,
            occurred_at: Option<DateTime<Utc>>,
        ) -> hook0_client::Event<'a> {
            hook0_client::Event {
                event_id: &None,
                event_type: event.event_type(),
                payload: hook0_client::Payload::from_binary(to_vec(&event).unwrap().as_ref()),
                payload_content_type: "application/json",
                metadata: Some(vec![(
                    "hook0_version".to_owned(),
                    to_value(crate_version!()).unwrap(),
                )]),
                occurred_at,
                labels: event.labels(),
            }
        }

        match self {
            Self::OrganizationCreated(e) => to_event(e, None),
            Self::EventTypeCreated(e @ EventEventTypeCreated { created_at, .. }) => {
                to_event(e, Some(created_at))
            }
        }
    }
}

trait Event: std::fmt::Debug + Clone + Serialize {
    fn event_type(&self) -> &'static str;
    fn labels(&self) -> Vec<(String, Value)>;
}

#[derive(Debug, Clone, Serialize)]
pub struct EventOrganizationCreated {
    pub organization_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

impl Event for EventOrganizationCreated {
    fn event_type(&self) -> &'static str {
        "api.organization.created"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![]
    }
}

impl From<EventOrganizationCreated> for Hook0ClientEvent {
    fn from(e: EventOrganizationCreated) -> Self {
        Self::OrganizationCreated(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventEventTypeCreated {
    pub application_id: Uuid,
    pub service_name: String,
    pub resource_type_name: String,
    pub verb_name: String,
    pub event_type_name: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

impl Event for EventEventTypeCreated {
    fn event_type(&self) -> &'static str {
        "api.event_type.created"
    }

    fn labels(&self) -> Vec<(String, Value)> {
        vec![(
            "application_id".to_owned(),
            to_value(self.application_id).unwrap(),
        )]
    }
}

impl From<EventEventTypeCreated> for Hook0ClientEvent {
    fn from(e: EventEventTypeCreated) -> Self {
        Self::EventTypeCreated(e)
    }
}
