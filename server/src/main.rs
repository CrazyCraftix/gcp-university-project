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

    match translate_api.fetch_languages().await {
        Err(e) => {
            log::error!("could not fetch supported languages: {}", e);
            Err(actix_web::error::ErrorInternalServerError(e))
        }
        // serialize languages
        Ok(languages) => match serde_json::to_string(&languages) {
            Ok(serialized) => Ok(actix_web::HttpResponse::Ok().body(serialized)),
            Err(e) => {
                log::error!("could not serialize supported languages");
                Err(actix_web::error::ErrorInternalServerError(e))
            }
        },
    }
}

// translate text
#[actix_web::post("/translate")]
async fn translate(
    request: actix_web::web::Json<common::TranslationRequest>,
    translate_api: actix_web::web::Data<translate_api::TranslateApi>,
) -> Result<actix_web::HttpResponse, actix_web::error::Error> {
    log::info!("translating");

    let request = request.0;
    match translate_api.translate(request).await {
        Err(e) => {
            log::error!("could not translate: {}", e);
            Err(actix_web::error::ErrorInternalServerError(e))
        }
        Ok(translation) => {
            Ok(actix_web::HttpResponse::Ok().json(html_escape::decode_html_entities(&translation)))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // initialize translate api using environment variable: GOOGLE_APPLICATION_CREDENTIALS
    let translate_api = match std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
        Ok(path) => {
            match google_translate3::oauth2::read_service_account_key(&path).await {
                Ok(credentials) => match translate_api::TranslateApi::new(credentials).await {
                    Ok(translate_api) => Some(actix_web::web::Data::new(translate_api)),
                    Err(e) => {
                        log::error!("could not initialize translate api: {}", e);
                        None
                    }
                },
                Err(e) => {
                    log::error!("could not read google credentials from $GOOGLE_APPLICATION_CREDENTIALS={}: {}", path, e);
                    None
                }
            }
        }
        Err(e) => {
            log::warn!(
                "could not find GOOGLE_APPLICATION_CREDENTIALS environment variable: {}",
                e
            );
            None
        }
    };

    let web_root = std::env::var("WEB_ROOT").unwrap_or("/var/www".to_string());

    actix_web::HttpServer::new(move || {
        let mut app = actix_web::App::new();
        if let Some(translate_api) = translate_api.clone() {
            app = app.app_data(translate_api.clone());
        }
        app.service(languages)
            .service(translate)
            .service(proxy)
            .service(actix_files::Files::new("/", web_root.clone()).index_file("index.html"))
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}
