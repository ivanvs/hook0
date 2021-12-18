use log::error;
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json},
    Apiv2Schema,
};
use serde::Serialize;
use sqlx::query_as;
use uuid::Uuid;

use crate::iam::AuthProof;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Organization {
    pub organization_id: Uuid,
    pub role: String,
    pub name: String,
}

#[api_v2_operation(
    summary = "List organizations",
    description = "",
    operation_id = "organizations.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Organizations Management")
)]
pub async fn list(
    state: Data<crate::State>,
    auth: AuthProof,
) -> Result<Json<Vec<Organization>>, Hook0Problem> {
    struct OrganizationMetadata {
        name: String,
    }
    let mut organizations = vec![];

    for (organization_id, role) in auth.organizations() {
        let metadata = query_as!(
            OrganizationMetadata,
            "
                SELECT name
                FROM event.organization
                WHERE organization__id = $1
            ",
            &organization_id
        )
        .fetch_optional(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

        let name = metadata.map(|om| om.name).unwrap_or_else(|| {
            error!(
                "Could not find organization {} in database",
                &organization_id
            );
            organization_id.to_string()
        });

        let org = Organization {
            organization_id,
            role: role.to_string(),
            name,
        };

        organizations.push(org);
    }

    Ok(Json(organizations))
}
