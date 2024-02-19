// 🐻‍❄️💐 ume: Easy, self-hostable, and flexible image host made in Rust
// Copyright 2021-2024 Noel Towa <cutie@floofy.dev>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use axum::{
    extract::Path,
    http::{
        header::{self, AUTHORIZATION},
        HeaderValue, StatusCode,
    },
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::{headers::Header, TypedHeader};
use noelware_remi::StorageService;
use rand::distributions::{Alphanumeric, DistString};
use remi::{Blob, StorageService as _, UploadRequest};
use serde_json::{json, Value};

use super::extract::Multipart;

pub async fn main() -> Json<Value> {
    Json(json!({
        "hello": "world",
        "build_info": json!({
            "version": crate::version(),
            "commit": crate::COMMIT_HASH,
            "build_date": crate::BUILD_DATE
        })
    }))
}

pub async fn heartbeat() -> &'static str {
    "Ok."
}

pub struct UploaderKey(pub HeaderValue);
impl Header for UploaderKey {
    fn name() -> &'static axum::http::HeaderName {
        &AUTHORIZATION
    }

    fn encode<E: Extend<axum::http::HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(
            HeaderValue::from_bytes(self.0.as_ref()).unwrap(),
        ));
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum_extra::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        values
            .next()
            .map(|h| UploaderKey(h.clone()))
            .ok_or_else(axum_extra::headers::Error::invalid)
    }
}

#[instrument(name = "ume.image.get", skip_all)]
pub async fn get_image(
    Extension(storage): Extension<StorageService>,
    Path(image): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    if image.contains("..") {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "message": "route not found"
            })),
        ));
    }

    let Some(file) = storage
        .blob(&image)
        .await
        .inspect_err(|e| {
            error!(error = %e, %image, "unable to find the image specified");
            sentry::capture_error(&e);
        })
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "internal server error, pls try again later"
                })),
            )
        })?
    else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "message": "image doesn't exist?"
            })),
        ));
    };

    // we should never reach here, if we do then it is a problem we need to face
    if let Blob::File(mut file) = file {
        if file.content_type.is_none() {
            file.content_type = Some(remi_fs::default_resolver(&file.data));
        }

        let ct = file.content_type.unwrap();
        let mime = ct.parse::<mime::Mime>().map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "message": "unable to infer field data's contents"
                })),
            )
        })?;

        if mime.type_() != mime::IMAGE {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "message": "wanted a image from field data's contents, didn't receive one though..."
                })),
            ));
        }

        return Ok((
            [
                (header::CONTENT_TYPE, HeaderValue::from_str(&ct).unwrap()),
                (header::CONTENT_LENGTH, HeaderValue::from(file.size)),
            ],
            file.data,
        ));
    }

    unreachable!()
}

#[instrument(name = "ume.upload.image", skip_all)]
pub async fn upload_image(
    Extension(storage): Extension<StorageService>,
    Extension(config): Extension<crate::config::Config>,
    TypedHeader(UploaderKey(value)): TypedHeader<UploaderKey>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if value != config.uploader_key {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "invalid uploader key received"
            })),
        ));
    }

    let Some(field) = multipart
        .next_field()
        .await
        .inspect_err(|e| {
            error!(error = %e, "failed to get next multipart field");
            sentry::capture_error(&e);
        })
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "message": format!("multipart error: {}", super::extract::err_to_msg(&e)),
                    "details": super::extract::expand_details_from_err(&e)
                })),
            )
        })?
    else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "message": "was expecting a multipart field, but didn't receive anything"
            })),
        ));
    };

    let bytes = field
        .bytes()
        .await
        .inspect_err(|e| {
            error!(error = %e, "unable to get bytes from field");
            sentry::capture_error(&e);
        })
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "message": format!("multipart error: {}", super::extract::err_to_msg(&e)),
                    "details": super::extract::expand_details_from_err(&e)
                })),
            )
        })?;

    let content_type = remi_fs::default_resolver(bytes.as_ref());
    let mime = content_type.parse::<mime::Mime>().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "unable to infer field data's contents"
            })),
        )
    })?;

    if mime.type_() != mime::IMAGE {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "wanted a image from field data's contents, didn't receive one though..."
            })),
        ));
    }

    let ext = match mime.subtype() {
        mime::PNG => "png",
        mime::JPEG => "jpg",
        mime::GIF => "gif",
        mime::SVG => "svg",
        name => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "message": format!("cannot process {name} as a image")
                })),
            ))
        }
    };
    let name = format!(
        "{}.{ext}",
        Alphanumeric.sample_string(&mut rand::thread_rng(), 6)
    );

    info!(file = %name, "uploading image...");
    storage
        .upload(
            format!("./{name}"),
            UploadRequest::default()
                .with_content_type(Some(mime.to_string()))
                .with_data(bytes),
        )
        .await
        .inspect_err(|e| {
            error!(error = %e, file = %name, "unable to upload file");
            sentry::capture_error(&e);
        })
        .map(|()| {
            Json(json!({
                "filename": format!("{}images/{}", config.base_url, name)
            }))
        })
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "received unknown error pls try again later :<"
                })),
            )
        })
}
