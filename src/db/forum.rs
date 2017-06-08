use bincode;
use chrono;

use redis;
use cdrs;
use postgres;
use mongodb;


use bson;

use uuid::{Uuid,UuidVersion};

use super::Error;
use super::{BinaryData,ServerID,UserID,Date,ThreadID,PostID,Category};
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

const POST_EXPIRATION:usize=30;

pub struct Forum {
    redis_global:RedisCollection,
    redis_posts:RedisCollection,
    //redis_hot_threads:RedisCollection,
    //redis_threads_watchers:RedisCollection,
    postgres_session:PostgresSession,
    cassandra_session:CassandraSession,
    cassandra_create_post_query:BodyResResultPrepared,
    cassandra_get_post_query:BodyResResultPrepared,
    //mongo_users:MongoCollection,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Post{
    pub author:UserID,
    //pub last_edit_date:Date,
    pub message:String,
}

pub struct Thread{
    pub id:ThreadID,
    pub author:UserID,
    pub caption:String,
    pub category:Category,
}


//pub struct

impl Forum {
    pub fn new(
        redis_global_client:&RedisClient,
        redis_posts_client:&RedisClient,
        //redis_hot_threads_client:&RedisClient,
        mongo_db:&MongoDatabase
    ) -> Result<Self,Error> {
        let redis_global = redis_global_client.get_collection()?;
        let redis_posts = redis_posts_client.get_collection()?;
        //let redis_hot_threads = redis_hot_threads_client.get_collection()?;
        //let redis_threads_watchers = redis_threads_watchers_client.get_collection()?;
        let mut cassandra_session = Self::connect_to_cassandra()?;
        let cassandra_create_post_query=Self::prepare_cassandra_create_post_query(&mut cassandra_session)?;
        let cassandra_get_post_query=Self::prepare_cassandra_get_post_query(&mut cassandra_session)?;
        let postgres_session = Self::connect_to_postgres()?;
        //let mongo_users = mongo_db.get_collection("users");

        let mut forum=Forum{
            redis_global,
            redis_posts,
            //redis_hot_threads,
            //redis_threads_watchers,
            postgres_session,
            cassandra_session,
            cassandra_create_post_query,
            cassandra_get_post_query,
        };

        Ok( forum )
    }

    fn connect_to_cassandra() -> Result<CassandraSession,Error> {
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


        Ok( cassandra_session )
    }

    fn connect_to_postgres() -> Result<PostgresSession,Error> {
        let tls_mode = postgres::TlsMode::None;
        let postgres_session = PostgresSession::connect("postgresql://postgresql_user:user@localhost/users",tls_mode)?;

        Ok( postgres_session )
    }

    fn prepare_cassandra_create_post_query(cassandra_session:&mut CassandraSession) -> Result<BodyResResultPrepared,Error> {
        let create_post_cql = "INSERT INTO posts
            (id, thread_id, author, date, last_edit, message)
            VALUES (?, ?, ?, ?, ?, ?)";

        let create_post_query_prepared = cassandra_session.prepare(create_post_cql.to_string(), true, true)?.
            get_body()?.into_prepared().unwrap();

        Ok( create_post_query_prepared )
    }

    fn prepare_cassandra_get_post_query(cassandra_session:&mut CassandraSession) -> Result<BodyResResultPrepared,Error> {
        let get_post_cql = "SELECT author, last_edit, message FROM posts WHERE id=?";

        let get_post_query_prepared = cassandra_session.prepare(get_post_cql.to_string(), true, true)?.
            get_body()?.into_prepared().unwrap();

        Ok( get_post_query_prepared )
    }

    fn uuid_to_value(id:&Uuid) -> Value {
        Value::new_normal(cdrs::types::value::Bytes::new( Vec::from(&id.as_bytes()[..]) ))
    }

    fn date_to_value(date:&Date) -> Value {
        let naive_date=date.naive_utc();
        let base=chrono::naive::date::NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0);//like UNIX
        let milliseconds_cassandra = naive_date.signed_duration_since(base).num_milliseconds();

        milliseconds_cassandra.into()
    }

    pub fn add_post(&mut self, thread:ThreadID, author:UserID, date:Date, message:String) -> Result<PostID,Error> {
        let id=Uuid::new(UuidVersion::Random).unwrap();

        let values: Vec<Value> = vec![
            Self::uuid_to_value(&id),
            Self::uuid_to_value(&thread),
            author.into(),
            Self::date_to_value(&date),
            Self::date_to_value(&date),
            message.into()
        ];

        let execution_params = CassandraParams::new(CassandraConsistency::One)
            .values(values)
            .finalize();

        let executed = self.cassandra_session.execute(&self.cassandra_create_post_query.id, execution_params, false, false)?.get_body()?;

        let insert_result=self.postgres_session.execute(
            "INSERT INTO posts (id, thread_id, author, date) VALUES($1, $2, $3, $4)",
            &[&id,&thread,&author,&date.naive_utc()]
        )?;

        if insert_result==1 {
            return Ok( id );
        }else{
            return Err(Error::Other("Can not add post".to_string()));
        }
    }

    pub fn create_thread(&mut self, users:&Users, author:UserID, category:Category, caption:String, message:String) -> Result<ThreadID,Error> {
        let date=UTC::now();

        loop{
            let id=Uuid::new(UuidVersion::Random).unwrap();

            let insert_result=self.postgres_session.execute(
                "INSERT INTO threads (id, author, category, caption) VALUES($1, $2, $3, $4)",
                &[&id,&author,&category,&caption]
            )?;

            if insert_result==1 {
                self.add_post(id,author,date,message)?;
                users.add_thread(author, id)?;
                return Ok(id);
            }
        }
    }

    pub fn get_post(&mut self, id:PostID) -> Result<Option<Post>,Error> {
        use cdrs::types::ByName;
        let data:Option<BinaryData> = self.redis_posts.get(id.to_string())?;

        match data {
            Some( data ) => {
                let post:Post=bincode::deserialize(&data)?;
                return Ok( Some(post) );
            },
            None => {},
        }

        let execution_params = CassandraParams::new(CassandraConsistency::One)
            .values(vec![Self::uuid_to_value(&id)])
            .finalize();

        let result_body = self.cassandra_session.execute(&self.cassandra_get_post_query.id, execution_params, false, false)?.get_body()?;
        let result_rows = result_body.into_rows().unwrap();

        if result_rows.len()>0 {
            let row=&result_rows[0];

            let author:UserID=row.r_by_name("author").unwrap();
            //let last_edit_date:Date=row.r_by_name("last_edit").unwrap();
            let message:String=row.r_by_name("message").unwrap();

            let post=Post{
                author,
                //last_edit_date,
                message,
            };

            let data:BinaryData=bincode::serialize(&post,bincode::Bounded(4024))?;
            self.redis_posts.set_ex(id.to_string(), data, POST_EXPIRATION)?;

            Ok( Some(post) )
        }else{
            Ok( None )
        }
    }

    pub fn get_thread_by_id(&self,id:ThreadID) -> Result<Option<Thread>,Error> {
        let result_rows=self.postgres_session.query(
            "SELECT author, caption, category FROM threads WHERE id=$1",
            &[&id]
        )?;

        if result_rows.len()>0 {
            let row=result_rows.iter().next().unwrap();

            let thread = Thread {
                id: id,
                author: row.get(0),
                caption: row.get(1),
                category: row.get(2)
            };

            Ok(Some(thread))
        }else{
            Ok(None)
        }
    }

    pub fn get_threads(&self,category:Category) -> Result<Vec<Thread>,Error> {
        let result_rows=self.postgres_session.query(
            "SELECT id, author, caption FROM threads WHERE category=$1",
            &[&category]
        )?;

        let mut threads=Vec::with_capacity(128);

        for row in &result_rows {
            let thread = Thread {
                id: row.get(0),
                author: row.get(1),
                caption: row.get(2),
                category: category,
            };

            threads.push(thread);
        }

        Ok(threads)
    }

    pub fn get_all_post_ids_for_thread(&mut self, thread:ThreadID) -> Result<Vec<PostID>,Error> {
        let result_rows=self.postgres_session.query(
            "SELECT id FROM posts WHERE thread_id=$1",
            &[&thread]
        )?;

        let mut post_ids=Vec::with_capacity(128);

        for row in &result_rows {
            let post_id=row.get(0);
            post_ids.push(post_id);
        }

        Ok(post_ids)
    }

    pub fn get_post_ids_for_thread_by_author(&mut self, thread:ThreadID, author:UserID) -> Result<Vec<PostID>,Error> {
        let result_rows=self.postgres_session.query(
            "SELECT id FROM posts WHERE thread_id=$1 AND author=$2",
            &[&thread,&author]
        )?;

        let mut post_ids=Vec::with_capacity(128);

        for row in &result_rows {
            let post_id=row.get(0);
            post_ids.push(post_id);
        }

        Ok(post_ids)
    }

    pub fn clear(&mut self) -> Result<(),Error> {
        use cdrs::query::Query;

        self.postgres_session.execute("DELETE FROM threads",&[])?;
        self.postgres_session.execute("DELETE FROM posts",&[])?;

        let delete_query = CassandraQuery::new("TRUNCATE posts").finalize();
        self.cassandra_session.query(delete_query, true, true)?;

        Ok(())
    }
}
