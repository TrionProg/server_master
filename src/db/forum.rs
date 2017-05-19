use bincode;

use redis;
use cdrs;
use postgres;
use mongodb;


use bson;

use uuid::{Uuid,UuidVersion};

use super::Error;
use super::{BinaryData,ServerID,UserID};
use super::{RedisClient,MongoDatabase};
use super::{RedisCollection,MongoCollection,CassandraSession,PostgresSession};

use std::time::Instant as TimeInstant;
use postgres::types::ToSql;
use redis::Commands;
use cdrs::query::QueryBuilder as CassandraQuery;
use cdrs::query::QueryParamsBuilder as CassandraParams;
use cdrs::consistency::Consistency as CassandraConsistency;
use cdrs::frame::frame_result::BodyResResultPrepared;
use cdrs::types::value::Value;
use bson::Bson;

use super::Users;

const USER_EXPIRATION:usize=30;

pub struct Forum {
    redis_forum:RedisCollection,
    redis_global:RedisCollection,
    postgres_session:PostgresSession,
    cassandra_session:CassandraSession,
    cassandra_create_post_query:BodyResResultPrepared,
    cassandra_add_post_query:BodyResResultPrepared,
    //mongo_users:MongoCollection,
}

pub enum Category {
    About,
    Talk
}

impl Forum {
    pub fn new(redis_forum_client:&RedisClient, redis_global_client:&RedisClient, mongo_db:&MongoDatabase) -> Result<Self,Error> {
        let redis_forum = redis_forum_client.get_collection()?;
        let redis_global = redis_global_client.get_collection()?;
        let (cassandra_session, cassandra_create_post_query, cassandra_add_post_query) = Self::connect_to_cassandra()?;
        let postgres_session = Self::connect_to_postgres()?;
        //let mongo_users = mongo_db.get_collection("users");

        let mut forum=Forum{
            redis_forum,
            redis_global,
            //cassandra_session,
            postgres_session,
            cassandra_session,
            cassandra_create_post_query,
            cassandra_add_post_query,
        };

        Ok( forum )
    }

    fn connect_to_cassandra() -> Result<(CassandraSession, BodyResResultPrepared, BodyResResultPrepared),Error> {
        let cassandra_address="127.0.0.1:9042";
        let cassandra_transport = match cdrs::transport::TransportTcp::new(cassandra_address) {
            Ok( cassandra_transport ) => cassandra_transport,
            Err( e ) => return Err(Error::CassandraConnectionError( Box::new(e) )),
        };
        let cassandra_client = cdrs::client::CDRS::new(cassandra_transport, cdrs::authenticators::NoneAuthenticator);
        let mut cassandra_session = cassandra_client.start(cdrs::compression::Compression::None)?;

        cassandra_session.query(
            cdrs::query::QueryBuilder::new("USE master_server;").finalize(),
            true,
            true
        )?;

        let (create_post_query_prepared, add_post_query_prepared)=Self::prepare_cassandra(&mut cassandra_session)?;

        Ok( (cassandra_session, create_post_query_prepared, add_post_query_prepared) )
    }

    fn connect_to_postgres() -> Result<PostgresSession,Error> {
        let tls_mode = postgres::TlsMode::None;
        let postgres_session = PostgresSession::connect("postgresql://postgresql_user:user@localhost/users",tls_mode)?;

        Ok( postgres_session )
    }

    fn prepare_cassandra(cassandra_session:&mut CassandraSession) -> Result<(BodyResResultPrepared,BodyResResultPrepared),Error> {
        let create_post_cql = "INSERT INTO user_keyspace.users
            (id, author, date, last_edit, message, previous, next)
            VALUES (?, ?, ?, ?, ?, NULL, NULL)";

        let create_post_query_prepared = cassandra_session.prepare(create_post_cql.to_string(), true, true)?.
            get_body()?.into_prepared().unwrap();

        let add_post_cql = "INSERT INTO user_keyspace.users
            (id, author, date, last_edit, message, previous, next)
            VALUES (?, ?, ?, ?, ?, ?, NULL)";

        let add_post_query_prepared = cassandra_session.prepare(add_post_cql.to_string(), true, true)?.
            get_body()?.into_prepared().unwrap();

        Ok( (create_post_query_prepared, add_post_query_prepared) )
    }

    pub fn create_first_post(&mut self, users:&mut Users, author:UserID, message:String) -> Result<Uuid,Error> {
        let id=Uuid::new(UuidVersion::Random).unwrap();

        let date=TimeInstant::now();

        let values: Vec<Value> = vec![
            id.to_string().into(),
            author.into(),
            date.to_string().into(),
            date.to_string().into(),
            message.into()
        ];

        let execution_params = CassandraParams::new(CassandraConsistency::One)
            .values(values)
            .finalize();

        let executed = self.cassandra_session.execute(&self.cassandra_create_post_query.id, execution_params, false, false)?;

        println!("executed:\n{:?}", executed);

        Ok( id )
    }
}
/*
    pub fn create_thread(&mut self, users:&mut Users, user_id:UserID, category:Category, caption:String, message:String) -> Result<Uuid,Error> {
        loop{
            let id=Uuid::new(UuidVersion::Random).unwrap();

            let insert_result=self.postgres_session.execute(
                "INSERT INTO threads (id, author, first_post, data) VALUES($1, $2, current_date, $3)",
                &[&id,&author,&data]
            )?;

            if insert_result==1 {
                return Ok(id);
            }
        }
*/
