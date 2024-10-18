
pub async fn get<T: serde::de::DeserializeOwned>(url: &str, token: Option<&str>) -> Result<T, reqwest::Error> {
    let base = url::Url::parse(&web_sys::window().unwrap().location().href().unwrap()).unwrap();
    let req = reqwest::Client::new()
        .get(base.join(url).unwrap());
    let req = if let Some(token) = token {
        req.header("Authorization", format!("Bearer {}", token))
    } else {
        req
    };

    let res = req.send().await?.json().await?;
    Ok(res)
}

pub async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(url: &str, body: B, token: Option<&str>) -> Result<T, reqwest::Error> {
    let base = url::Url::parse(&web_sys::window().unwrap().location().href().unwrap()).unwrap();
    let req = reqwest::Client::new()
        .post(base.join(url).unwrap())
        .json(&body);
    let req = if let Some(token) = token {
        req.header("Authorization", format!("Bearer {}", token))
    } else {
        req
    };

    let res = req.send().await?.json().await?;
    Ok(res)
}
