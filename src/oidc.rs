use crate::{config::OidcConfiguration, util::Zeroizing};
use color_eyre::eyre::{self, OptionExt};
use openidconnect::{
    ClientId, IssuerUrl,
    core::{CoreClient, CoreProviderMetadata, CoreUserInfoClaims},
    reqwest::{self, redirect},
};
use poem_openapi::auth::Bearer;
use std::{collections::HashSet, sync::Arc};

type OidcClient = openidconnect::Client<
    openidconnect::EmptyAdditionalClaims,
    openidconnect::core::CoreAuthDisplay,
    openidconnect::core::CoreGenderClaim,
    openidconnect::core::CoreJweContentEncryptionAlgorithm,
    openidconnect::core::CoreJsonWebKey,
    openidconnect::core::CoreAuthPrompt,
    openidconnect::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    openidconnect::StandardTokenResponse<
        openidconnect::IdTokenFields<
            openidconnect::EmptyAdditionalClaims,
            openidconnect::EmptyExtraTokenFields,
            openidconnect::core::CoreGenderClaim,
            openidconnect::core::CoreJweContentEncryptionAlgorithm,
            openidconnect::core::CoreJwsSigningAlgorithm,
        >,
        oauth2::basic::BasicTokenType,
    >,
    openidconnect::StandardTokenIntrospectionResponse<
        openidconnect::EmptyExtraTokenFields,
        oauth2::basic::BasicTokenType,
    >,
    oauth2::StandardRevocableToken,
    openidconnect::StandardErrorResponse<openidconnect::RevocationErrorResponseType>,
    openidconnect::EndpointSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointMaybeSet,
    openidconnect::EndpointMaybeSet,
>;

#[derive(Clone)]
pub struct OidcService {
    authorized_groups: Arc<HashSet<String>>,
    client: OidcClient,
    http_client: reqwest::Client,
}

impl OidcService {
    pub async fn from_config(config: &OidcConfiguration) -> eyre::Result<Self> {
        let http_client = reqwest::Client::builder()
            .redirect(redirect::Policy::none())
            .build()?;

        let url = IssuerUrl::new(config.url.clone())?;
        let metadata = CoreProviderMetadata::discover_async(url, &http_client).await?;

        let client_id = ClientId::new(config.client_id.clone());
        let client = CoreClient::from_provider_metadata(metadata, client_id, None);

        Ok(Self {
            authorized_groups: Arc::new(config.authorized_groups.clone()),
            client,
            http_client,
        })
    }

    async fn load_user_info(&self, access_token: &Bearer) -> eyre::Result<CoreUserInfoClaims> {
        let access_token = oauth2::AccessToken::new(access_token.token.clone());
        let claims = self
            .client
            .user_info(access_token, None)?
            .request_async(&self.http_client)
            .await?;

        Ok(claims)
    }

    pub async fn load_username(&self, access_token: &Bearer) -> eyre::Result<Zeroizing<String>> {
        let info = self.load_user_info(access_token).await?;
        let username = info
            .preferred_username()
            .ok_or_eyre("no username in token")?
            .to_string();

        Ok(Zeroizing(username))
    }

    pub async fn is_allowed(&self, access_token: &Bearer) -> eyre::Result<()> {
        let user_info = self.load_user_info(access_token).await?;

        todo!("check if the user is member of any of the mentioned groups: {user_info:?}");

        Ok(())
    }
}
