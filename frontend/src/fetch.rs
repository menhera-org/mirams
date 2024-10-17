
pub async fn get<U: reqwest::IntoUrl, T: serde::de::DeserializeOwned>(url: &str) -> Result<T, reqwest::Error> {
    let base = url::Url::parse(&web_sys::window().unwrap().location().href().unwrap()).unwrap();
    let res = reqwest::get(base.join(url).unwrap()).await?
        .json()
        .await?;
    Ok(res)
}

pub async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(url: &str, body: B) -> Result<T, reqwest::Error> {
    let base = url::Url::parse(&web_sys::window().unwrap().location().href().unwrap()).unwrap();
    let res = reqwest::Client::new()
        .post(base.join(url).unwrap())
        .json(&body)
        .send()
        .await?
        .json()
        .await?;
    Ok(res)
}
