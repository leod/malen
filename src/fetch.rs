use thiserror::Error;

use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, Request, RequestInit, RequestMode, Response};

#[derive(Error, Debug, Clone)]
pub enum FetchError {
    #[error("new request error: {0:?}")]
    NewRequest(JsValue),

    #[error("fetch error: {0:?}")]
    Fetch(JsValue),

    #[error("get result: {0:?}")]
    GetResult(JsValue),

    #[error("await result: {0:?}")]
    AwaitResult(JsValue),
}

pub async fn fetch(path: &str) -> Result<Response, FetchError> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::SameOrigin);

    let request = Request::new_with_str_and_init(path, &opts).map_err(FetchError::NewRequest)?;

    let response: Response = {
        let window = web_sys::window().unwrap();
        let promise = window.fetch_with_request(&request);
        let value = JsFuture::from(promise).await.map_err(FetchError::Fetch)?;
        assert!(value.is_instance_of::<Response>());
        value.dyn_into().unwrap()
    };

    Ok(response)
}

pub async fn fetch_array_buffer(path: &str) -> Result<ArrayBuffer, FetchError> {
    let response = fetch(path).await?;

    let array_buffer: ArrayBuffer = {
        let promise = response.array_buffer().map_err(FetchError::GetResult)?;
        let value = JsFuture::from(promise)
            .await
            .map_err(FetchError::AwaitResult)?;
        assert!(value.is_instance_of::<ArrayBuffer>());
        value.dyn_into::<ArrayBuffer>().unwrap()
    };

    Ok(array_buffer)
}

pub async fn fetch_blob(path: &str) -> Result<Blob, FetchError> {
    let response = fetch(path).await?;

    let blob: Blob = {
        let promise = response.blob().map_err(FetchError::GetResult)?;
        let value = JsFuture::from(promise)
            .await
            .map_err(FetchError::AwaitResult)?;
        assert!(value.is_instance_of::<Blob>());
        value.dyn_into::<Blob>().unwrap()
    };

    Ok(blob)
}

pub async fn fetch_data(path: &str) -> Result<Vec<u8>, FetchError> {
    let array_buffer = fetch_array_buffer(path).await?;
    let typed_buffer = Uint8Array::new(&array_buffer);
    let mut data = vec![0; typed_buffer.length() as usize];
    typed_buffer.copy_to(&mut data[..]);

    Ok(data)
}
