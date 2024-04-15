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

    pub async fn fetch_languages(&self) -> Result<Vec<common::Language>, google_translate3::Error> {
        self.hub
            .projects()
            .get_supported_languages("projects/dhbw-cloud-computing/locations/global")
            .display_language_code("en")
            .doit()
            .await
            .map(|res| {
                if let Some(languages) = res.1.languages {
                    languages
                        .iter()
                        .filter_map(|language| {
                            if let (
                                Some(language_code),
                                Some(display_name),
                                Some(true),
                                Some(true),
                            ) = (
                                &language.language_code,
                                &language.display_name,
                                language.support_source,
                                language.support_target,
                            ) {
                                Some(common::Language {
                                    code: language_code.into(),
                                    display_name: display_name.into(),
                                })
                            } else {
                                None
                            }
                        })
                        .collect()
                } else {
                    vec![]
                }
            })
    }

    pub async fn translate(
        &self,
        translation_request: common::TranslationRequest,
    ) -> Result<String, google_translate3::Error> {
        let mut request = google_translate3::api::TranslateTextRequest::default();
        request.source_language_code = Some(translation_request.source_language_code);
        request.target_language_code = Some(translation_request.target_language_code);
        request.contents = Some(vec![translation_request.text]);

        self.hub
            .projects()
            .translate_text(request, "projects/dhbw-cloud-computing/locations/global")
            .doit()
            .await
            .map(|res| match res.1.translations {
                Some(translations) => match translations.get(0) {
                    Some(translation) => match &translation.translated_text {
                        Some(text) => text.into(),
                        None => "".into(),
                    },
                    None => "".into(),
                },
                None => "".into(),
            })
    }
}
