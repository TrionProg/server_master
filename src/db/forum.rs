use bincode;
use chrono;

use redis;
use cdrs;
use postgres;
use mongodb;


use bson;

use uuid::{Uuid,UuidVersion};

use super::Error;
use super::{BinaryData,ServerID,UserID,Date,ThreadID};
use super::{RedisClient,MongoDatabase};
use super::{RedisCollection,MongoCollection,CassandraSession,PostgresSession};

use postgres::types::ToSql;
use redis::{Commands, PipelineCommands};

use cdrs::query::QueryBuilder as CassandraQuery;
use cdrs::query::QueryParamsBuilder as CassandraParams;
use cdrs::consistency::Consistency as CassandraConsistency;
use cdrs::frame::frame_result::BodyResResultPrepared;
use cdrs::types::value::Value;
use cdrs::types::value::ValueType;

use chrono::offset::utc::UTC;

use byteorder::{LittleEndian, WriteBytesExt};

use bson::Bson;

use super::Users;

const THREAD_WATCHERS_COUNT_SIZE:usize=4;
const THREAD_WATCHERS_USERID_SIZE:usize=4;
const CATEGORIES_NUMBER:usize=2;

const USER_EXPIRATION:usize=30;

pub struct Forum {
    redis_global:RedisCollection,
    //redis_posts:RedisCollection,
    redis_hot_threads:RedisCollection,
    //redis_threads_watchers:RedisCollection,
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

//pub struct

impl Forum {
    pub fn new(
        redis_global_client:&RedisClient,
        redis_hot_threads_client:&RedisClient,
        mongo_db:&MongoDatabase
    ) -> Result<Self,Error> {
        let redis_global = redis_global_client.get_collection()?;
        let redis_hot_threads = redis_hot_threads_client.get_collection()?;
        //let redis_threads_watchers = redis_threads_watchers_client.get_collection()?;
        let (cassandra_session, cassandra_create_post_query, cassandra_add_post_query) = Self::connect_to_cassandra()?;
        let postgres_session = Self::connect_to_postgres()?;
        //let mongo_users = mongo_db.get_collection("users");

        let mut forum=Forum{
            redis_global,
            redis_hot_threads,
            //redis_threads_watchers,
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
        let create_post_cql = "INSERT INTO posts
            (id, author, date, last_edit, message, previous, next)
            VALUES (?, ?, ?, ?, ?, NULL, NULL)";

        let create_post_query_prepared = cassandra_session.prepare(create_post_cql.to_string(), true, true)?.
            get_body()?.into_prepared().unwrap();

        let add_post_cql = "INSERT INTO posts
            (id, author, date, last_edit, message, previous, next)
            VALUES (?, ?, ?, ?, ?, ?, NULL)";

        let add_post_query_prepared = cassandra_session.prepare(add_post_cql.to_string(), true, true)?.
            get_body()?.into_prepared().unwrap();

        Ok( (create_post_query_prepared, add_post_query_prepared) )
    }

//    fn load_watchers(&self) -> Result<(),Error> {
//        for i in 0..CATEGORIES_NUMBER {


    fn uuid_to_value(id:&Uuid) -> Value {
        Value::new_normal(cdrs::types::value::Bytes::new( Vec::from(&id.as_bytes()[..]) ))
    }

    fn date_to_value(date:&Date) -> Value {
        let naive_date=date.naive_utc();
        let base=chrono::naive::date::NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0);//like UNIX
        let milliseconds_cassandra = naive_date.signed_duration_since(base).num_milliseconds();

        milliseconds_cassandra.into()
    }


    pub fn create_first_post(&mut self, author:UserID, date:Date, message:String) -> Result<Uuid,Error> {
        let id=Uuid::new(UuidVersion::Random).unwrap();

        let values: Vec<Value> = vec![
            Self::uuid_to_value(&id),
            author.into(),
            Self::date_to_value(&date),
            Self::date_to_value(&date),
            message.into()
        ];

        println!("{:?}",values);

        let execution_params = CassandraParams::new(CassandraConsistency::One)
            .values(values)
            .finalize();

        let executed = self.cassandra_session.execute(&self.cassandra_create_post_query.id, execution_params, false, false)?.get_body()?;

        println!("executed:\n{:?}", executed);

        Ok( id )
    }
/*
    pub fn add_post(&mut self, thread:ThreadID, author:UserID, date:Date, message:String) -> Result<Uuid,Error> {
        let id=Uuid::new(UuidVersion::Random).unwrap();
/*
        let (new_val,) : (isize,) = try!(redis::transaction(&self.redis_threads, &[thread], |pipe| {
            let
            let prev_post
    let old_val : isize = try!(con.get(key));
    pipe
        .set(key, old_val + 1).ignore()
        .get(key).query(&con)
}));
*/

        let values: Vec<Value> = vec![
            Self::uuid_to_value(&id),
            author.into(),
            Self::date_to_value(&date),
            Self::date_to_value(&date),
            message.into()
        ];
    }
    */

    pub fn create_thread(&mut self, users:&mut Users, author:UserID, category:Category, caption:String, message:String) -> Result<Uuid,Error> {
        let date=UTC::now();

        let first_post=self.create_first_post(author, date, message)?;

        loop{
            let id=Uuid::new(UuidVersion::Random).unwrap();

            let insert_result=self.postgres_session.execute(
                "INSERT INTO threads (id, author, category, caption, first_post, last_post, first_post_date, last_post_date) VALUES($1, $2, $5, $5, $6, $6)",
                &[&id,&author,&category.to_i32(),&caption,&first_post,&date]
            )?;

            if insert_result==1 {
                //self.add_thread_to_watchers(id, author)?;
                return Ok(id);
            }
        }
    }
/*
    fn add_thread_to_watchers(&self, id:Uuid, author:UserID) -> Result<(),Error> {
        let watchers_count:u32=1;
        //let mut thread_watchers:BinaryData=Vec::with_capacity(THREAD_WATCHERS_COUNT_SIZE+THREAD_WATCHERS_USERID_SIZE);
        //thread_watchers.write_u32::<LittleEndian>(watchers_count)?;
        //thread_watchers.write_i32::<LittleEndian>(author)?;
        let mut thread_watchers=Vec::with_capacity(1);
        thread_watchers.push(author);

        self.redis_threads_watchers.set(id.to_string(),thread_watchers)?;

        let (new_val,) : (isize,) = try!(redis::transaction(&con, &[key], |pipe| {
    let old_val : isize = try!(con.get(key));
    pipe
        .set(key, old_val + 1).ignore()
        .get(key).query(&con)
}));

        Ok(())
    }
*/
    //pub fn make_thread_hot(&mut self, id:Uuid) -> Result<(),Error> {

}

impl Category {
    pub fn to_i32(&self) -> i32 {
        match *self {
            Category::About => 0,
            Category::Talk => 1,
        }
    }
}
