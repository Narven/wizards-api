use async_std::sync::RwLock;
use femme;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tide::http::cookies::Cookie;
use tide::{Next, Request, Response, Result};

#[derive(serde::Deserialize, Serialize)]
struct Wizard {
    name: String,
    level: u8,
}

#[derive(Deserialize, Serialize)]
struct NameParams {
    pub name: String,
}

impl Default for NameParams {
    fn default() -> Self {
        Self {
            name: "World".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct State {
    name: String,
}

struct Repository {
    wizards: HashMap<String, Wizard>,
}

impl Repository {
    fn new() -> Self {
        Self {
            wizards: HashMap::new(),
        }
    }
}

fn check_req<'a>(
    req: Request<()>,
    next: Next<'a, ()>,
) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
    Box::pin(async {
        if req.header("Authorization").is_none() {
            let mut res = Response::new(403);
            res.set_body("Forbidden");
            return Ok(res);
        }
        Ok(next.run(req).await)
    })
}

async fn create(mut req: Request<State>) -> tide::Result {
    let wizard: Wizard = req.body_json().await?;
    let state = req.state();
    println!("{:?}", state);
    let res = Response::builder(tide::StatusCode::Created)
        .body(format!("{} is level {}", wizard.name, wizard.level))
        .build();
    Ok(res)
}

async fn handle_name(req: Request<()>) -> tide::Result<String> {
    let name: NameParams = req.query()?;
    Ok(format!("Hello, {}", name.name))
}

async fn cookie_handler(req: Request<()>) -> tide::Result {
    let name = req.cookie("name").unwrap();

    let mut res = Response::new(200);
    res.set_body(format!("hello, {}", name.value()));

    res.insert_cookie(Cookie::new("app", "tide"));
    res.remove_cookie(Cookie::new("name", "foo"));
    Ok(res)
}

async fn read_all(_req: Request<()>) -> tide::Result<tide::Body> {
    let wizards = vec![
        Wizard {
            name: "Gandaf".to_string(),
            level: 100,
        },
        Wizard {
            name: "merlin".to_string(),
            level: 10,
        },
    ];

    Ok(tide::Body::from_json(&wizards)?)
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    femme::start();

    let repository = Arc::new(Repository::new());

    let mut app = tide::with_state(repository);

    app.with(tide::log::LogMiddleware::new());

    app.at("/").get(handle_name);

    app.at("/cookies").get(cookie_handler);

    app.at("/www").serve_dir("./www")?;

    app.at("/wizards").nest({
        let mut api = tide::new();
        api.at("/").get(read_all);
        api.at("/1").get(create);

        api
    });
    app.listen("0.0.0.0:8080").await?;
    Ok(())
}
