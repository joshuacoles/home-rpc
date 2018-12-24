#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use rocket::http::Status;
use rocket_contrib::json::Json;

use rocket::fairing::AdHoc;
use rocket::State;

mod exec;

use crate::exec::{ScheduleData, ExecData, ExecContext};

#[post("/exec", data = "<data>")]
fn exec(context: State<ExecContext>, data: Json<ExecData>) -> Result<Status, String> {
    match exec::execute(&context, data.into_inner()) {
        Ok(_) => Ok(Status::Ok),
        Err(e) => Err(e.description()),
    }
}

#[post("/schedule", data = "<schedule_data>")]
fn schedule(context: State<ExecContext>, schedule_data: Json<ScheduleData>) -> Result<Status, String> {
    match exec::schedule(&context, schedule_data.into_inner()) {
        Ok(_) => Ok(Status::Ok),
        Err(e) => Err(e.description()),
    }
}

#[get("/alive")]
fn alive() -> &'static str {
    "Alive"
}

fn main() {
    let base_uri = std::env::vars().find(|(k, _)| k == "APP_ROOT")
        .map_or("/".to_string(), |(_, v)| v);

    rocket::ignite()
        .mount(base_uri.as_str(), routes![
            exec,
            schedule,

            alive,
        ])
        .attach(AdHoc::on_attach("Assets Config", |rocket| {
            let scripts_dir = rocket.config()
                .get_str("scripts_dir")
                .unwrap_or("/scripts/")
                .to_string();

            Ok(rocket.manage(ExecContext(scripts_dir.into())))
        }))
        .launch();
}
