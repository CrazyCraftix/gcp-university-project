use std::fmt::Display;

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

impl Display for SendRequestError {
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
        .await.map_err(|e| SendRequestError(e))?;
    Ok(actix_web::HttpResponse::build(client_response.status())
        .streaming(client_response.take_payload()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .app_data(actix_web::web::Data::new(awc::Client::new()))
            .service(proxy)
            .service(actix_files::Files::new("/", "./dist").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}