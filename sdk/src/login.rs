use crate::{api_url::*, error::*, prelude::*};
use hyper::{body::HttpBody, header::SET_COOKIE, Body, HeaderMap, Method, Request};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Login {
    pub flag: bool,
    pub msg: String,
    pub data: Option<Value>,
    pub code: Option<i32>,
}

// 登录到 Natayark ID OAuth2
pub async fn login_oauth2(account: &Account, api_client: &mut Client) -> Result<Login> {
    // 创建 Headers
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("Cookie", api_client.cookies.to_string().parse()?);

    // 克隆 API Client 中的 Hyper Client
    let client = api_client.client.clone();

    // 创建对应 API 的 POST 请求
    let mut req = Request::builder().method(Method::POST).uri(OAUTH2_URL);

    // 添加 Headers
    req.headers_mut().unwrap().extend(headers);

    // 添加 Body
    let req = req.body(Body::from(serde_json::to_string(account).unwrap()))?;

    // 用 Hyper Client 发送 Request
    let mut res = client.request(req).await?;
    let headers = res.headers();

    // 添加 Cookie
    headers
        .get_all(SET_COOKIE)
        .iter()
        .for_each(|c| api_client.cookies.add_cookie(c.to_str().unwrap()).unwrap());

    let json: Login = serde_json::from_slice(&res.data().await.unwrap()?.to_vec()).unwrap();

    if !json.flag {
        return Err(Error::new(json.code.unwrap_or(-1), &json.msg));
    }

    Ok(json)
}

// 通过 Natayark ID 回调，获取 Code
pub async fn oauth2_callback(_login_res: Login, api_client: &mut Client) -> Result<String> {
    // 创建 Headers
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("Cookie", api_client.cookies.to_string().parse()?);

    // 克隆 API Client 中的 Hyper Client
    let client = api_client.client.clone();

    // 创建对应 API 的 POST 请求
    let mut req = Request::builder().method(Method::POST).uri(OAUTH2_CALLBACK);

    // 添加 Headers
    req.headers_mut().unwrap().extend(headers);

    // 添加 Body
    let req = req.body(Body::empty())?;

    // 用 Hyper Client 发送 Request
    let mut res = client.request(req).await?;
    let headers = res.headers();

    // 添加 Cookie
    headers
        .get_all(SET_COOKIE)
        .iter()
        .for_each(|c| api_client.cookies.add_cookie(c.to_str().unwrap()).unwrap());

    let json: Login = serde_json::from_slice(&res.data().await.unwrap()?.to_vec()).unwrap();

    if !json.flag {
        return Err(Error::new(json.code.unwrap_or(-1), &json.msg));
    } else {
        match json.data {
            Some(data) => {
                return Ok(data["code"].as_str().unwrap().to_string());
            }
            _ => todo!(),
        }
    }
}

pub async fn login_by_code(code: String, api_client: &mut Client) -> Result<()> {
    // 创建 Headers
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("Cookie", api_client.cookies.to_string().parse()?);

    // 克隆 API Client 中的 Hyper Client
    let client = api_client.client.clone();

    // 创建对应 API 的 POST 请求
    let mut req =
        Request::builder()
            .method(Method::POST)
            .uri(format!("{}{}", LOGIN_CALLBACK, code.as_str()));

    // 添加 Headers
    req.headers_mut().unwrap().extend(headers);

    // 添加 Body
    let req = req.body(Body::empty())?;

    // 用 Hyper Client 发送 Request
    let mut res = client.request(req).await?;
    let headers = res.headers().clone();

    // 添加 Cookie
    headers
        .get_all(SET_COOKIE)
        .iter()
        .for_each(|c| api_client.cookies.add_cookie(c.to_str().unwrap()).unwrap());
    let data = res.data().await.unwrap()?;
    // println!("data: {:#?}", String::from_utf8(data.to_vec()).unwrap());
    let json: Login = serde_json::from_slice(&data.to_vec()).unwrap();

    // Auth
    let mut auth = Auth::new();

    if !json.flag {
        return Err(Error::new(json.code.unwrap_or(-1), &json.msg));
    } else {
        // 把 Authorization 写入 auth
        auth.authorization = headers
            .get("Authorization")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        match json.data {
            Some(data) => {
                auth.session = data.as_str().unwrap().to_string();
            }
            _ => todo!(),
        }
    }
    api_client.auth = auth;
    Ok(())
}

pub async fn login(account: &Account,api_client:&mut Client) -> Result<()>{
    let login_oa2 = login_oauth2(account, api_client).await?;
    let code = oauth2_callback(login_oa2, api_client).await?;
    login_by_code(code, api_client).await?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_login() -> Result<()> {
        let account = Account::new("example@example.com", "password");
        let mut client = Client::new();
        login(&account, &mut client).await?;
        println!("auth: {:#?}", client.auth);
        Ok(())
    }
}