#![deny(warnings)]

use warp::Filter;
use serde::{Deserialize, Serialize};
use futures::future;
use std::str::FromStr;
use tokio_postgres::Error;
use uuid::Uuid;
use std::net::{IpAddr, Ipv6Addr};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Account {
    pub uuid: Uuid,
    pub username: String,
    pub avatar: String,
    pub pubkeys: Vec<String>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Message {
    pub from: Uuid,
    pub to: Uuid,
    pub tag: String,
    pub base64: String
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let config = tokio_postgres::Config::from_str("host=/run/postgresql user=merklechat dbname=merklechat").unwrap();
    let pool = database::create_pool(config);

    database::setup(pool.clone()).await?;

    let route_help = warp::path("help").
        and_then(handlers::help);
    let route_getaccount = warp::path("getaccount").
        and(database::with_pool(pool.clone())).
        and(warp::path::param()).
        and_then(handlers::getaccount);
    let route_getmessages = warp::path("getmessages").
        and(database::with_pool(pool.clone())).
        and(warp::path::param()).
        and_then(handlers::getmessages);

    let route_register = warp::path("register").and(warp::path::end()).
        and(database::with_pool(pool.clone())).
        and(warp::body::json()).and_then(handlers::register);
    let route_sendmessage = warp::path("sendmessage").and(warp::path::end()).
        and(database::with_pool(pool.clone())).
        and(warp::body::json()).and_then(handlers::sendmessage);

    let routes = warp::get().and(route_help.or(route_getaccount).or(route_getmessages)).or(
        warp::post().and(route_register.or(route_sendmessage)));

    let cors = warp::cors().allow_any_origin().allow_methods(vec!["POST", "GET", "OPTION"]).allow_header("content-type");
    let log = warp::log("server::requests");
    warp::serve(routes.with(log).with(cors)).run((IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 3030)).await;

    Ok(())
}

mod database {
    use uuid::Uuid;
    use tokio_postgres::Error;
    use mobc_postgres::{tokio_postgres, PgConnectionManager};
    use tokio_postgres::NoTls;
    use warp::Filter;
    pub type Pool = mobc::Pool<PgConnectionManager<NoTls>>;
//    pub type Connection = mobc::Connection<PgConnectionManager<NoTls>>;

    pub fn with_pool(pool:Pool)
                     -> impl warp::Filter<Extract = (Pool,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || pool.clone())
    }


    pub fn create_pool(config:tokio_postgres::Config) -> Pool {
        let manager = PgConnectionManager::new(config, NoTls);
        let pool = Pool::builder().max_open(10).build(manager);
        return pool
    }


    pub async fn get_account(pool:Pool, uuid:Uuid) -> Result<crate::Account, Error> {
        let client = pool.get().await.unwrap();
        let (row, rows) = crate::future::try_join(
            client.query_one("SELECT username, avatar      FROM   account WHERE id = $1", &[&uuid]),
            client.query    ("SELECT encode(key, 'base64') FROM publickey WHERE id = $1", &[&uuid])).await?;
        let keys = rows.iter().map(|r| r.get(0)).collect();

        Ok(crate::Account{uuid: uuid, username: row.get(0), avatar: row.get(1), pubkeys: keys})
    }


    pub async fn set_account(pool:Pool, account:crate::Account) -> Result<(), Error> {
        let client = pool.get().await.unwrap();
        client.execute("INSERT INTO account (id, avatar, username) VALUES ($1, $2, $3)", &[&account.uuid, &account.avatar, &account.username]).await?;
        let qry = client.prepare("INSERT INTO publickey (id, key) VALUES ($1, decode($2, 'base64'))").await?;
        for k in account.pubkeys.into_iter() {
            client.execute(&qry, &[&account.uuid, &k]).await?;
        }
        Ok(())
    }


    pub async fn get_messages(pool:Pool, uuid:Uuid) -> Result<Vec<crate::Message>, Error> {
        let client = pool.get().await.unwrap();
        let rows = client.query("SELECT \"from\", encode(tag, 'base64'), translate(encode(message, 'base64'), E'\n', '') FROM message WHERE \"to\" = $1", &[&uuid]).await?;
        let messages = rows.iter().map(|r| crate::Message{from: r.get(0), to: uuid.clone(), tag: r.get(1), base64: r.get(2)}).collect();
        Ok(messages)
    }


    pub async fn set_message(pool:Pool, message:crate::Message) -> Result<(), Error> {
        let client = pool.get().await.unwrap();
        client.execute("INSERT INTO message (\"from\", \"to\", tag, message) VALUES ($1, $2, decode($3, 'base64'), decode($4, 'base64'))",
                       &[&message.from, &message.to, &message.tag, &message.base64]).await?;
        Ok(())
    }


    pub async fn setup(pool:Pool) -> Result<(), Error> {
        let client = pool.get().await.unwrap();
        crate::future::try_join3(
            client.execute("
            CREATE TABLE IF NOT EXISTS account (
                id UUID PRIMARY KEY,
                avatar TEXT,
                username TEXT
            )", &[]),
            client.execute("
            CREATE TABLE IF NOT EXISTS publickey (
                id  UUID,
                key BYTEA,
                FOREIGN KEY (id) REFERENCES account(id)
            )", &[]),
            client.execute("
            CREATE TABLE IF NOT EXISTS message (
                \"from\"  UUID,
                \"to\"    UUID,
                tag       BYTEA,
                message   BYTEA,
                FOREIGN KEY (\"from\") REFERENCES account(id),
                FOREIGN KEY (\"to\")   REFERENCES account(id)
            )", &[])).await?;
        Ok(())
    }
}

mod handlers {
    use uuid::Uuid;
    use std::convert::Infallible;
    use crate::database;

    pub async fn help() -> Result<impl warp::Reply, Infallible> {
        Ok("test")
    }

    pub async fn getaccount(pool:crate::database::Pool, uuid:Uuid) -> Result<impl warp::Reply, warp::Rejection> {
        match database::get_account(pool, uuid).await {
            Ok(account) => Ok(warp::reply::json(&account)),
            Err(_) => Err(warp::reject::not_found())
        }
    }

    pub async fn getmessages(pool:crate::database::Pool, uuid:Uuid) -> Result<impl warp::Reply, warp::Rejection> {
        let messages = database::get_messages(pool, uuid).await.unwrap();
        Ok(warp::reply::json(&messages))
    }

    pub async fn register(pool:crate::database::Pool, account: crate::Account) -> Result<impl warp::Reply, warp::Rejection> {
        database::set_account(pool, account).await.unwrap();
        Ok("OK")
    }

    pub async fn sendmessage(pool:crate::database::Pool, message: crate::Message) -> Result<impl warp::Reply, warp::Rejection> {
        database::set_message(pool, message).await.unwrap();
        Ok("OK")
    }
}
