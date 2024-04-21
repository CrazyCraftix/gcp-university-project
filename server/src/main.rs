use redis::Commands as _;

mod translate_api;

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
    redis_client: actix_web::web::Data<Option<redis::Client>>,
) -> Result<actix_web::HttpResponse, actix_web::error::Error> {
    log::info!("translating");

    let request = request.0;

    // connect to cache
    let mut cache_connection = match redis_client.as_ref() {
        Some(redis_client) => {
            match redis_client.get_connection_with_timeout(std::time::Duration::from_millis(100)) {
                Ok(connection) => Some(connection),
                Err(e) => {
                    log::error!("couldn't connect to cache: {}", e);
                    None
                }
            }
        }
        None => {
            log::info!("cache unavailable");
            None
        }
    };

    // check cache
    let request_hash = request.generate_hash();
    if let Some(connection) = &mut cache_connection {
        if let Ok(translation) = connection.get::<u64, String>(request_hash) {
            log::info!("cache hit!");
            return Ok(actix_web::HttpResponse::Ok().json(translation));
        }
        log::info!("cache miss!");
    }

    // cache miss -> request translation from api
    match translate_api.translate(request).await {
        Err(e) => {
            log::error!("could not translate: {}", e);
            Err(actix_web::error::ErrorInternalServerError(e))
        }
        Ok(translation) => {
            let translation = html_escape::decode_html_entities(&translation);

            // save translation to cache
            if let Some(connection) = &mut cache_connection {
                match connection.set::<u64, String, String>(request_hash, translation.to_string()) {
                    Ok(_) => log::info!("translation cached"),
                    Err(_) => log::info!("error caching translation"),
                }
            }

            Ok(actix_web::HttpResponse::Ok().json(translation))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // parse GOOGLE_APPLICATION_CREDENTIALS environment variable and initialize translate api
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
    if translate_api.is_none() {
        log::warn!("translate api unavailable");
    }

    // parse REDIS_CONNECTION environment variable
    let redis_client = match std::env::var("REDIS_CONNECTION") {
        Ok(connection_string) => match redis::Client::open(connection_string) {
            Ok(client) => Some(client),
            Err(e) => {
                log::warn!("invalid redis connection details: {}", e);
                None
            }
        },
        Err(e) => {
            log::error!(
                "could not find REDIS_CONNECTION environment variable: {}",
                e
            );
            None
        }
    };
    if redis_client.is_none() {
        log::warn!("redis cache unavailable");
    }
    let redis_client = actix_web::web::Data::new(redis_client);

    // parse WEB_ROOT environment variable
    let web_root = std::env::var("WEB_ROOT").unwrap_or("/var/www".to_string());

    // parse BIND_ADDRESS environment variable
    let bind_address: core::net::SocketAddr = std::env::var("BIND_ADDRESS")
        .ok()
        .map(|address_string| match address_string.parse() {
            Ok(address) => Some(address),
            Err(e) => {
                log::error!("could not parse bind address: {}", e);
                None
            }
        })
        .flatten()
        .unwrap_or(std::net::SocketAddr::V4(std::net::SocketAddrV4::new(
            std::net::Ipv4Addr::new(0, 0, 0, 0),
            80,
        )));
    log::info!("bind address: {}", bind_address);

    actix_web::HttpServer::new(move || {
        let mut app = actix_web::App::new();
        if let Some(translate_api) = translate_api.clone() {
            app = app.app_data(translate_api.clone());
        }
        app.app_data(redis_client.clone())
            .service(languages)
            .service(translate)
            .service(actix_files::Files::new("/", web_root.clone()).index_file("index.html"))
    })
    .bind(bind_address)?
    .run()
    .await
}
