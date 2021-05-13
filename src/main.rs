#![feature(duration_consts_2)]

use std::{
    env, time,
};
use std::sync::RwLock;
use lazy_static::lazy_static;
use actix_web::{get, web, HttpServer, App, HttpRequest, Responder, Error, HttpResponse};
use std::future::{
    Ready, ready,
};
use serde::{
    Serialize, Deserialize,
};
use async_std::task;

static EXPIRE_TIME: time::Duration = time::Duration::new(3600, 0);
lazy_static! {
    static ref ACCESS_TOKEN: RwLock<String> = RwLock::new(String::new());
    static ref CREATED_TIME: RwLock<time::Instant> = RwLock::new(time::Instant::now() - time::Duration::new(7200, 0));
}

#[derive(Serialize)]
struct RespondToken {
    token: String,
}

impl RespondToken {
    const fn new(token: String) -> Self {
        Self {
            token,
        }
    }
}

impl Responder for RespondToken {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();

        ready(Ok(
            HttpResponse::Ok()
                .content_type("application/json")
                .body(body)
        ))
    }
}

#[derive(Deserialize, Debug)]
struct WxTokenRespond {
    access_token: String,
    expires_in: usize,
}

async fn renew_token<'a>(appid: &'a str, secret: &'a str) -> Result<(), ()> {
    const ENDPOINT: &str = "https://api.weixin.qq.com/cgi-bin/token";
    let client = reqwest::Client::new();
    let res = client
        .get(ENDPOINT)
        .query(&[
            ("grant_type", "client_credential"),
            ("appid", appid),
            ("secret", secret),
        ])
        .send()
        .await
        .unwrap();
    let data = res.json::<WxTokenRespond>().await.unwrap();
    *(ACCESS_TOKEN.write().unwrap()) = data.access_token;
    Ok(())
}

#[get("/token")]
async fn fetch_token(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {

    let query_str = req.query_string();
    let qs = qstring::QString::from(query_str);
    let key = qs.get("key").unwrap_or_default();
    if key.ne(data.secret.as_str()) {
        return Err(HttpResponse::Forbidden());
    }

    let now_time = time::Instant::now();
    if now_time.checked_duration_since(*(CREATED_TIME.read()).unwrap()).unwrap().ge(&EXPIRE_TIME) {
        task::block_on(renew_token(&data.wx_appid.as_str(), &data.wx_secret.as_str())).unwrap();
        *(CREATED_TIME.write().unwrap()) = time::Instant::now();
    }

    Ok(RespondToken::new((*(ACCESS_TOKEN.read().unwrap())).clone()))
}

struct AppState {
    secret: String,
    wx_appid: String,
    wx_secret: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .data(AppState {
                secret: env::var("SERVER_SECRET").unwrap(),
                wx_appid: env::var("WX_APPID").unwrap(),
                wx_secret: env::var("WX_SECRET").unwrap(),
            })
            .service(fetch_token)
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
