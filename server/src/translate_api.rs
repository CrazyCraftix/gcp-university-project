pub struct TranslateApi {
    hub: google_translate3::Translate<
        google_translate3::hyper_rustls::HttpsConnector<
            google_translate3::hyper::client::HttpConnector,
        >,
    >,
}

impl TranslateApi {
    pub async fn new(
        service_account_key: google_translate3::oauth2::ServiceAccountKey,
    ) -> Result<TranslateApi, std::io::Error> {
        let auth = google_translate3::oauth2::ServiceAccountAuthenticator::builder(
            service_account_key.clone(),
        )
        .build()
        .await?;

        let hub = google_translate3::Translate::new(
            google_translate3::hyper::Client::builder().build(
                google_translate3::hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_only()
                    .enable_http1()
                    .build(),
            ),
            auth,
        );

        Ok(TranslateApi { hub })
    }

    pub async fn fetch_languages(
        &self,
    ) -> Result<Option<Vec<google_translate3::api::SupportedLanguage>>, google_translate3::Error>
    {
        self.hub
            .projects()
            .locations_get_supported_languages("projects/dhbw-cloud-computing/locations/global")
            .display_language_code("en")
            .doit()
            .await
            .map(|res| res.1.languages)
    }
}
