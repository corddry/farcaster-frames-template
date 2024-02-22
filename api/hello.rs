use farcaster_frames_template::{get_character, APIError, FrameActionPayload};
use vercel_runtime::{
    http::bad_request, process_request, process_response, run_service, service_fn, Body,
    Error, Request, RequestPayloadExt, Response, ServiceBuilder, StatusCode,
};

use image::{DynamicImage, imageops};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber
        ::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        .init();

    // This allows to extend the tower service with more layers
    let handler = ServiceBuilder::new()
        .map_request(process_request)
        .map_response(process_response)
        .service(service_fn(handler));

    run_service(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    match req.method().as_str() {
        "GET" => handle_get_request(req).await,
        "POST" => handle_post_request(req).await,
        _ => {
            let msg = "Method Not Allowed";
            tracing::warn!(msg);
            Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::from(msg))?)
        }
    }
}

pub async fn handle_get_request(req: Request) -> Result<Response<Body>, Error> {

    let head: u16 = 282;
    let body: u16 = 25;

    let sprite_base_url = "https://fusioncalc.com/wp-content/themes/twentytwentyone/pokemon/custom-fusion-sprites-main/CustomBattlers/";

    let fusion1_image = format!(
        "{}{}.{}.png",
        sprite_base_url,
        head,
        body
    );
    

    let frame_image = fusion1_image;
        // "https://upload.wikimedia.org/wikipedia/commons/6/6c/Star_Wars_Logo.svg";
    let frame_post_url = req.uri();
    let html_content = format!(
        r#"<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta property="og:image" content="{0}" /> 
                <meta property="fc:frame" content="vNext" />
                <meta property="fc:frame:post_url" content="{1}" />
                <meta property="fc:frame:image" content="{0}" />
                <meta property="fc:frame:button:1" content="Woah, Cool Fusion!" />
                <title>Farcaster Frames Template</title>
            </head>
            <body>
                <h1>Woah, Cool Fusion!</h1>
            </body>
            </html>"#,
        frame_image, frame_post_url
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(html_content.into())?)
}

pub async fn handle_post_request(req: Request) -> Result<Response<Body>, Error> {
    let payload = req.payload::<FrameActionPayload>();

    // TODO: Check if frame request is valid (using body.trustedData?.messageBytes)

    match payload {
        Err(err) => {
            tracing::error!("Invalid payload {}", err);
            bad_request(APIError {
                message: "Invalid payload",
                code: "invalid_payload",
            })
        }
        Ok(None) => {
            tracing::error!("No payload");
            bad_request(APIError {
                message: "No payload",
                code: "no_payload",
            })
        }
        Ok(Some(payload)) => {
            tracing::info!("Payload: {}", payload);

            let character =
                get_character(payload.untrusted_data.fid).unwrap_or("".to_string());
            let frame_image = format!(
                "https://placehold.co/600x400/black/yellow?text={}",
                character
            );
            let html_content = format!(
                r#"<!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta property="og:image" content="{0}" /> 
                    <meta property="fc:frame" content="vNext" />
                    <meta property="fc:frame:image" content="{0}" />
                    <title>Farcaster Frames Template</title>
                </head>
                </html>"#,
                frame_image
            );

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(html_content.into())?)
        }
    }
}
