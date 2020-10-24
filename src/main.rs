// create table data(timestamp integer primary key, data text);

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate hex;

use rocket_contrib::databases::rusqlite;
use std::convert::TryInto;
use std::error::Error;

#[database("sqlite")]
struct Database(rusqlite::Connection);

struct Message {
    temp: f32,
    hum: f32,
    bat: f32,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/callback?<time>&<data>")]
fn callback(
    database: Database,
    time: String,
    data: String,
) -> Result<&'static str, Box<dyn Error>> {
    //bytes[0] << 8 + bytes[0];
    println!("Callback: {} {}", time, data);
    let time_num: i64 = time.parse()?;
    database.0.execute(
        "INSERT INTO data (timestamp, data) VALUES (?1, ?2)",
        &[&time_num, &data],
    )?;
    Ok("Callback")
}

fn main() {
    rocket::ignite()
        .attach(Database::fairing())
        .mount("/", routes![index, callback])
        .launch();
}

fn decode_message(data: String) -> Result<Message, Box<dyn Error>> {
    println!("data: {}", data);
    let bytes = hex::decode(data)?;
    println!("bytes: {} {} {}", bytes[0], bytes[1], bytes[2]);
    Ok(Message {
        temp: decode_float(&bytes[1..3].try_into().unwrap()),
        hum: decode_float(&bytes[3..5].try_into().unwrap()),
        bat: decode_float(&bytes[5..7].try_into().unwrap()),
    })
}

fn decode_float(bytes: &[u8; 2]) -> f32 {
    (((bytes[0] as i16) << 8) + (bytes[1] as i16)) as f32 / 100.0
}

#[test]
fn it_decodes_temp() {
    assert_eq!(
        decode_message("00000000000000".to_string()).unwrap().temp,
        0.0
    );
    assert_eq!(
        decode_message("00000100000000".to_string()).unwrap().temp,
        0.01
    );
    assert_eq!(
        decode_message("00111100000000".to_string()).unwrap().temp,
        43.69
    );
    assert_eq!(
        decode_message("00F6DA00000000".to_string()).unwrap().temp,
        -23.42
    );
}

#[test]
fn it_decodes_hum() {
    assert_eq!(
        decode_message("00000027100000".to_string()).unwrap().hum,
        100.0
    );
}

#[test]
fn it_decodes_bat() {
    assert_eq!(
        decode_message("000000000001F4".to_string()).unwrap().bat,
        5.0
    );
}
