use finchers::prelude::*;
use finchers::{path, routes};
use finchers_session::{session, Session};

use failure::Fallible;
use futures::prelude::*;
use http::{Response, StatusCode};
use log::info;
use redis::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
struct Login {
    username: String,
}

fn main() -> Fallible<()> {
    pretty_env_logger::init();

    let client = Client::open("redis://127.0.0.1/")?;
    let backend = finchers_session::backend::redis(client)
        .key_prefix("my-app-name")
        .cookie_name("my-session-id")
        .timeout(Duration::from_secs(60 * 3));
    let session = Arc::new(session(backend));

    let greet = path!(@get /)
        .and(session.clone())
        .and_then(|session: Session<Login, _>| {
            let response = match session.get() {
                Ok(Some(login)) => html(format!(
                    "Hello, {}! <br />\n\
                     <form method=\"post\" action=\"/logout\">\n\
                     <input type=\"submit\" value=\"Log out\" />\n\
                     </form>\
                     ",
                    login.username
                )),
                _ => Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header("content-type", "text/html; charset=utf-8")
                    .body("<a href=\"/login\">Log in</a>".into())
                    .unwrap(),
            };
            session.into_future().map(|_| response)
        });

    let login =
        path!(@get /"login"/)
            .and(session.clone())
            .and_then(|session: Session<Login, _>| {
                let response = match session.get() {
                    Ok(Some(_login)) => redirect_to("/").map(|_| ""),
                    _ => html(
                        "login form\n\
                         <form method=\"post\">\n\
                         <input type=\"text\" name=\"username\">\n\
                         <input type=\"submit\">\n\
                         </form>",
                    ),
                };
                session.into_future().map(|_| response)
            });

    let login_post = {
        #[derive(Debug, Deserialize)]
        struct Form {
            username: String,
        }

        path!(@post /"login"/)
            .and(session.clone())
            .and(endpoints::body::urlencoded())
            .and_then(|mut session: Session<Login, _>, form: Form| {
                session
                    .set(Login {
                        username: form.username,
                    }).into_future()
                    .and_then(move |()| session.into_future().map(|_| redirect_to("/")))
            })
    };

    let logout =
        path!(@post /"logout"/)
            .and(session.clone())
            .and_then(|mut session: Session<Login, _>| {
                session.remove();
                session.into_future().map(|_| redirect_to("/"))
            });

    let endpoint = endpoint::EndpointObj::new(routes![greet, login, login_post, logout,]);

    info!("Listening on http://127.0.0.1:4000");
    finchers::launch(endpoint).start("127.0.0.1:4000");

    Ok(())
}

fn redirect_to(location: &str) -> Response<()> {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("location", location)
        .body(())
        .unwrap()
}

fn html<T>(body: T) -> Response<T> {
    Response::builder()
        .header("content-type", "text/html; charset=utf-8")
        .body(body)
        .unwrap()
}
