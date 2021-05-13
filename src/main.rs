
#![feature(duration_consts_2)]

use std::{
    env, time
};
use std::sync::RwLock;
use lazy_static::lazy_static;
use actix_web::{get, HttpServer, App, HttpRequest, Responder, Error, HttpResponse};
use std::future::{
    Ready, ready,
};
use serde::{
    Serialize, Deserialize,
};
use std::ops::Sub;
use async_std::task;

static mut APP_ID: Option<String> = None;
static mut APP_SECRET: Option<String> = None;
static EXPIRE_TIME: time::Duration = time::Duration::new(3600, 0);
lazy_static! {
    static ref ACCESS_TOKEN: RwLock<String> = RwLock::new(String::new());
    static ref CREATED_TIME: RwLock<time::Instant> = RwLock::new(time::Instant::now() - time::Duration::new(7200, 0));
}

fn load_env() {
    unsafe {
        APP_ID = Option::from(env::var("WX_APPID").unwrap());
        APP_SECRET = Option::from(env::var("WX_SECRET").unwrap());
    }
}

#[derive(Serialize)]
struct RespondToken {
    token: String
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

async fn renew_token() -> Result<(), ()>{
    const ENDPOINT: &str = "https://api.weixin.qq.com/cgi-bin/token";
    let client = reqwest::Client::new();
    unsafe {
        let appid = APP_ID.clone().unwrap();
        let secret = APP_SECRET.clone().unwrap();
        let res = client
            .get(ENDPOINT)
            .query(&[
                ("grant_type", "client_credential"),
                ("appid", appid.as_str()),
                ("secret", secret.as_str()),
            ])
            .send()
            .await
            .unwrap();
        let data = res.json::<WxTokenRespond>().await.unwrap();
        *(ACCESS_TOKEN.write().unwrap()) = data.access_token;
    }
    Ok(())
}

#[get("/token")]
async fn fetch_token(req: HttpRequest) -> impl Responder {
    let now_time = time::Instant::now();
    if now_time.checked_duration_since(*(CREATED_TIME.read()).unwrap()).unwrap().ge(&EXPIRE_TIME) {
        task::block_on(renew_token());
        *(CREATED_TIME.write().unwrap()) = time::Instant::now();
    }

    RespondToken::new((*(ACCESS_TOKEN.read().unwrap())).clone())

}

#[actix_web::main]
async fn main() -> std::io::Result<()>  {
    load_env();
    // task::block_on(renew_token());

    HttpServer::new(|| {
        App::new()
            .service(fetch_token)
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
