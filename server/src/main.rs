mod translate_api;

use actix_web::HttpMessage as _;

#[derive(Debug)]
struct SendRequestError(awc::error::SendRequestError);

impl actix_web::ResponseError for SendRequestError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self.0 {
            awc::error::SendRequestError::Connect(awc::error::ConnectError::Timeout) => {
                actix_web::http::StatusCode::GATEWAY_TIMEOUT
            }
            awc::error::SendRequestError::Connect(_) => actix_web::http::StatusCode::BAD_REQUEST,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Display for SendRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[actix_web::get("/tutorial/data.json")]
async fn proxy(
    client: actix_web::web::Data<awc::Client>,
) -> Result<actix_web::HttpResponse, SendRequestError> {
    let mut client_response = client
        .get("https://yew.rs/tutorial/data.json")
        .send()
        .await
        .map_err(|e| SendRequestError(e))?;
    Ok(actix_web::HttpResponse::build(client_response.status())
        .streaming(client_response.take_payload()))
}

// retrieve available languages
#[actix_web::get("/languages")]
async fn languages(
    translate_api: actix_web::web::Data<translate_api::TranslateApi>,
) -> Result<actix_web::HttpResponse, actix_web::error::Error> {
    log::info!("retrieving supported languages");

    // fetch languages and map them to type common::Language
    let languages = match translate_api.fetch_languages().await {
        Err(e) => {
            log::error!("could not fetch supported languages: {}", e);
            Err(actix_web::error::ErrorInternalServerError(e))
        }
        Ok(None) => Ok(vec![]),
        Ok(Some(languages)) => Ok(languages
            .iter()
            .filter_map(|language| {
                if let (Some(language_code), Some(display_name), Some(true), Some(true)) = (
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
            .collect()),
    }?;

    // serialize languages
    match serde_json::to_string(&languages) {
        Ok(serialized) => Ok(actix_web::HttpResponse::Ok().body(serialized)),
        Err(e) => {
            log::error!("could not serialize supported languages");
            Err(actix_web::error::ErrorInternalServerError(e))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let credentials = google_translate3::oauth2::read_service_account_key(
        std::env::var("GOOGLE_CREDENTIALS").expect("environment variable GOOGLE_CREDENTIALS not set - it needs to contain the path to the json file with service account credentials!")
    )
    .await
    .expect("could not read service account credentials - does the environment variable GOOGLE_CREDENTIALS contain the path to a valid json file with service account credentials?");

    let translate_api = actix_web::web::Data::new(
        translate_api::TranslateApi::new(credentials)
            .await
            .expect("could not initialize translate api"),
    );

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(translate_api.clone())
            .service(languages)
            .service(proxy)
            .service(actix_files::Files::new("/", "./dist").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
