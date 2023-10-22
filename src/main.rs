use axum::{
    extract::{DefaultBodyLimit, Multipart},
    response::Html,
    routing::{get, post},
    Router,
};
use tower_http::limit::RequestBodyLimitLayer;

use base64::{engine::general_purpose, Engine as _};

use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, services::ServeDir};

use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "../templates/root.html")]
struct HomeTemplate {}

#[derive(TemplateOnce)]
#[template(path = "../templates/img.html")]
struct ImageUploadTemplate {
    images: Vec<Image>,
}

#[derive(Debug)]
struct Image {
    src: String,
    name: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv::dotenv().ok();

    let app = Router::new()
        .route("/", get(root))
        .route("/upload", post(upload_handler))
        .layer(CorsLayer::permissive())
        .layer(RequestBodyLimitLayer::new(999999999))
        .layer(DefaultBodyLimit::max(999999999))
        .nest_service("/static", ServeDir::new("./static/"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));

    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn upload_handler(mut multipart: Multipart) -> Html<String> {
    let mut image_bytes: Vec<Vec<u8>> = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let data = field.bytes().await.unwrap();

        image_bytes.push(data.to_vec());
    }

    let mut to_render = ImageUploadTemplate { images: Vec::new() };

    for image in image_bytes {
        let encoded = general_purpose::STANDARD_NO_PAD.encode(image);
        let img = Image {
            src: encoded,
            name: "test".to_string(),
        };
        println!("Image: {:?}", &img);

        to_render.images.push(img);
    }

    Html(to_render.render_once().unwrap())
}

async fn detect_text(_image: String) -> String {
    let client = reqwest::Client::new();
    // curl -X POST \
    //    -H "Authorization: Bearer $(gcloud auth print-access-token)" \
    //    -H "x-goog-user-project: PROJECT_ID" \
    //    -H "Content-Type: application/json; charset=utf-8" \
    //    -d @request.json \
    //    "https://vision.googleapis.com/v1/images:annotate"
    let res = client
        .post("https://api.ocr.space/parse/image")
        .form(&[
            ("apikey", "helloworld"),
            ("language", "eng"),
            ("isOverlayRequired", "true"),
            ("base64Image", &_image),
        ])
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    res
}

async fn root() -> Html<String> {
    let template = HomeTemplate {};

    Html(template.render_once().unwrap())
}
