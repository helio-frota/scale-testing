// The simplest loadtest example
mod oidc;
mod restapi;
mod website;

use crate::oidc::get_token;
use crate::restapi::{
    get_advisory, get_importer, get_oganizations, get_packages, get_products, get_sboms,
    get_vulnerabilities, search_packages,
};
use crate::website::{
    website_advisories, website_importers, website_index, website_openapi, website_packages,
    website_sboms,
};
use goose::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_scenario(
            scenario!("WebsiteUser")
                .set_weight(1)?
                .register_transaction(transaction!(setup_custom_client).set_on_start())
                // After each transactions runs, sleep randomly from 5 to 15 seconds.
                .set_wait_time(Duration::from_secs(5), Duration::from_secs(15))?
                .register_transaction(transaction!(website_index).set_name("/index"))
                .register_transaction(transaction!(website_openapi).set_name("/openapi"))
                .register_transaction(transaction!(website_sboms).set_name("/sboms"))
                .register_transaction(transaction!(website_packages).set_name("/packages"))
                .register_transaction(transaction!(website_advisories).set_name("/advisories"))
                .register_transaction(transaction!(website_importers).set_name("/importers")),
        )
        .register_scenario(
            scenario!("RestAPIUser")
                .set_weight(1)?
                .register_transaction(transaction!(setup_custom_client).set_on_start())
                // After each transactions runs, sleep randomly from 5 to 15 seconds.
                .set_wait_time(Duration::from_secs(5), Duration::from_secs(15))?
                .register_transaction(
                    transaction!(get_oganizations).set_name("/api/v1/organization"),
                )
                .register_transaction(transaction!(get_advisory).set_name("/api/v1/advisory"))
                .register_transaction(transaction!(get_importer).set_name("/api/v1/importer"))
                .register_transaction(transaction!(get_packages).set_name("/api/v1/purl"))
                .register_transaction(transaction!(search_packages).set_name("/api/v1/purl?q=curl"))
                .register_transaction(transaction!(get_products).set_name("/api/v1/product"))
                .register_transaction(transaction!(get_sboms).set_name("/api/v1/sbom"))
                .register_transaction(
                    transaction!(get_vulnerabilities).set_name("/api/v1/vulnerability"),
                ),
        )
        .execute()
        .await?;

    Ok(())
}

async fn setup_custom_client(user: &mut GooseUser) -> TransactionResult {
    use reqwest::{header, Client};

    let issuer_url: String = std::env::var("ISSUER_URL").unwrap();
    let client_id: String = std::env::var("CLIENT_ID").unwrap();
    let client_secret: String = std::env::var("CLIENT_SECRET").unwrap();

    let auth_token: String = get_token(issuer_url, client_id, client_secret);
    {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(&auth_token).unwrap(),
        );

        // Build a custom client.
        let builder = Client::builder()
            .default_headers(headers)
            .user_agent("loadtest-ua")
            .timeout(Duration::from_secs(30));

        // Assign the custom client to this GooseUser.
        user.set_client_builder(builder).await?;
        Ok(())
    }
}