use bincode;

use redis;
use cdrs;
use postgres;
use mongodb;


use bson;

use uuid::Uuid;

use super::Error;
use super::{BinaryData,ServerID,UserID,ImageID,ThreadID};
use super::{RedisClient,MongoDatabase};
use super::{RedisCollection,MongoCollection,CassandraSession,PostgresSession};

use postgres::types::ToSql;
use redis::Commands;
use bson::Bson;

const USER_EXPIRATION:usize=30;


#[derive(Serialize, Deserialize, Debug)]
pub struct ShortUserInformation{
    pub login:String,
    pub avatar:Uuid,
    pub rating:f32,
    pub online_status:OnlineStatus,
}

pub struct Users{
    redis_users:RedisCollection,
    redis_global:RedisCollection,
    postgres_session:PostgresSession,
    mongo_users:MongoCollection,
}

pub enum AddUserResult{
    UserExists,
    Success(UserID),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OnlineStatus{
    Offline,
    Online,
    Playing(ServerID),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Award{
    pub id:i32,
    pub name:String,
    pub icon:Uuid,
    pub description:String,
}

pub struct FullUserInformation {
    pub user_id:UserID,
    pub doc:bson::ordered::OrderedDocument,
}

impl Users {
    pub fn new(redis_users_client:&RedisClient, redis_global_client:&RedisClient, mongo_db:&MongoDatabase) -> Result<Self,Error> {
        let redis_users = redis_users_client.get_collection()?;
        let redis_global = redis_global_client.get_collection()?;
        //let cassandra_session = Self::connect_to_cassandra()?;
        let postgres_session = Self::connect_to_postgres()?;
        let mongo_users = mongo_db.get_collection("users");

        let users=Users{
            redis_users,
            redis_global,
            //cassandra_session,
            postgres_session,
            mongo_users,
        };

        Ok( users )
    }

    fn connect_to_cassandra() -> Result<CassandraSession,Error> {
        let cassandra_address="127.0.0.1:9042";
        let cassandra_transport = match cdrs::transport::TransportTcp::new(cassandra_address) {
            Ok( cassandra_transport ) => cassandra_transport,
            Err( e ) => return Err(Error::CassandraConnectionError( Box::new(e) )),
        };
        let cassandra_client = cdrs::client::CDRS::new(cassandra_transport, cdrs::authenticators::NoneAuthenticator);
        let mut cassandra_session = cassandra_client.start(cdrs::compression::Compression::None)?;

        /*
        cassandra_session.query(
            cdrs::query::QueryBuilder::new("USE users;").finalize(),
            true,
            true
        )?;
        */

        Ok( cassandra_session )
    }

    fn connect_to_postgres() -> Result<PostgresSession,Error> {
        let tls_mode = postgres::TlsMode::None;
        let postgres_session = PostgresSession::connect("postgresql://postgresql_user:user@localhost/users",tls_mode)?;

        Ok( postgres_session )
    }


    pub fn add_user(&self, login:&str, password:&str) -> Result<AddUserResult,Error> {
        let exists_result_rows=self.postgres_session.query(
            "SELECT EXISTS(SELECT 1 FROM users WHERE login=$1)",
            &[&login]
        )?;

        if exists_result_rows.len()>0 {
            let exists:bool=exists_result_rows.get(0).get(0);

            if exists {
                return Ok(AddUserResult::UserExists);
            }
        }

        let default_avatar_small_b:BinaryData=match self.redis_global.get("default_avatar_small")? {
            Some( data ) => data,
            None => panic!("no \"default_avatar_small\" in redis"),
        };
        let default_avatar_small:Uuid=bincode::deserialize(&default_avatar_small_b)?;

        let default_avatar_big_b:BinaryData=match self.redis_global.get("default_avatar_big")? {
            Some( data ) => data,
            None => panic!("no \"default_avatar_big\" in redis"),
        };
        let default_avatar_big:Uuid=bincode::deserialize(&default_avatar_big_b)?;

        let insert_result=self.postgres_session.execute(
            "INSERT INTO users (login,password,avatar,rating) VALUES ($1, $2, $3, 0.0)",
            &[&login,&password,&default_avatar_small]
        )?;

        if insert_result!=1 {
            return Ok(AddUserResult::UserExists);
        }

        let user_id=match self.get_user_id_by_login(login)?{
            Some( user_id ) => user_id,
            None => return Ok(AddUserResult::UserExists),
        };

        let default_avatar_big_s=default_avatar_big.to_string();

        let doc = doc! {
            "_id" => user_id,
            "avatar" => default_avatar_big_s,
            "awards" => [],
            "history" => [],
            "mods" => [],
            "threads" => []
        };

        let concern=mongodb::common::WriteConcern{
            w:1,//TODO:change to number of hosts
            w_timeout:100,
            j:true,
            fsync:true,
        };

        self.mongo_users.insert_one(doc, Some(concern))?;

        Ok( AddUserResult::Success(user_id) )
    }

    pub fn get_user_id_by_login(&self, login:&str) -> Result<Option<UserID>, Error> {
        let result_rows=self.postgres_session.query(
            "SELECT id FROM users WHERE login=$1",
            &[&login]
        )?;

        if result_rows.len()>0 {
            let user_id:UserID=result_rows.get(0).get(0);

            Ok( Some(user_id) )
        }else{
            Ok( None )
        }
    }

    pub fn get_short_user_information_by_id(&self, user_id:UserID) -> Result<Option<ShortUserInformation>,Error> {
        let data:Option<BinaryData> = self.redis_users.get(user_id)?;

        match data {
            Some( data ) => {
                let user:ShortUserInformation=bincode::deserialize(&data)?;
                return Ok( Some(user) );
            },
            None => {},
        }

        /*
        let result_rows=self.postgres_session.query(
            "SELECT * FROM get_short_user_information($1) AS (login character(32), avatar uuid, rating real)",
            &[&user_id]
        )?;
        */

        let result_rows=self.postgres_session.query(
            "SELECT * FROM get_short_user_information($1)",
            &[&user_id]
        )?;

        if result_rows.len()>0 {
            let login_raw:String=result_rows.get(0).get(0);
            let login=login_raw.trim().to_string();
            let avatar:Uuid=result_rows.get(0).get(1);
            let rating:f32=result_rows.get(0).get(2);
            let online_status=OnlineStatus::Offline;

            let user=ShortUserInformation{
                login,
                avatar,
                rating,
                online_status
            };

            let data:BinaryData=bincode::serialize(&user,bincode::Bounded(96))?;
            self.redis_users.set_ex(user_id, data, USER_EXPIRATION)?;

            Ok( Some(user) )
        }else{
            Ok( None )
        }
    }

    pub fn user_exists_by_id(&self, user_id:UserID) -> Result<bool,Error> {
        let redis_exists:bool=self.redis_users.exists(user_id)?;

        if redis_exists {
            return Ok(true);
        }

        let exists_result_rows=self.postgres_session.query(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id=$1)",
            &[&user_id]
        )?;

        if exists_result_rows.len()>0 {
            let exists:bool=exists_result_rows.get(0).get(0);

            if exists {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn get_full_user_information_by_id(&self, user_id:UserID) -> Result<Option<FullUserInformation>,Error> {
        let find_filter=doc! {
            "_id" => user_id
        };

        let find_option=mongodb::coll::options::FindOptions{
            max_time_ms:Some( 25 ),
            ..Default::default()
        };

        let doc=self.mongo_users.find_one(Some(find_filter),Some(find_option))?;

        //println!("{:?}",doc);

        match doc {
            Some( doc ) => {
                let full_information=FullUserInformation{
                    user_id:user_id,
                    doc:doc,
                };

                Ok( Some(full_information) )
            },
            None => Ok( None ),
        }
    }

    pub fn give_award(&self, user_id:UserID, award_name:&str, description:String) -> Result<Option<usize>,Error> {
        let result_rows=self.postgres_session.query(
            "SELECT id,icon FROM awards WHERE name=$1",
            &[&award_name]
        )?;

        if result_rows.len()>0 {
            let postgres_award=result_rows.get(0);
            let id:i32=postgres_award.get(0);
            let icon:Uuid=postgres_award.get(1);

            let award=Award{
                id:id,
                name:String::from(award_name),
                icon:icon,
                description:description,
            };

            let serialized_award = bson::to_bson(&award)?;

            let find_filter=doc! {
                "_id" => user_id
            };

            let update_doc=doc! {
                "$push" => { "awards" => serialized_award }
            };

            self.mongo_users.find_one_and_update( find_filter ,update_doc, None );//TODO:None

            Ok( Some(id as usize) )
        }else{
            Ok( None )
        }
    }

    pub fn add_thread(&self, user_id:UserID, thread:ThreadID) -> Result<(),Error> {
        let find_filter=doc! {
            "_id" => user_id
        };

        let thread_s=thread.to_string();

        let update_doc=doc! {
            "$push" => { "threads" => thread_s }
        };

        self.mongo_users.find_one_and_update( find_filter ,update_doc, None );//TODO:None

        Ok(())
    }

    pub fn get_user_ids(&self) -> Result<Vec<UserID>,Error> {
        let result_rows=self.postgres_session.query(
            "SELECT id FROM users",
            &[]
        )?;

        let mut user_ids=Vec::with_capacity(128);

        for row in &result_rows {
            let user_id=row.get(0);
            user_ids.push(user_id);
        }

        Ok(user_ids)
    }

    pub fn clear(&mut self) -> Result<(),Error>{
        self.postgres_session.execute("DELETE FROM users",&[])?;
        self.mongo_users.drop()?;

        Ok(())
    }

    //modify award (chande icon and description

}

impl FullUserInformation {
    pub fn get_awards(&self) -> Result<Vec<Award>,Error> {
        match self.doc.get("awards") {
            Some(&Bson::Array(ref mongo_awards)) => {
                if mongo_awards.len()==0 {
                    return Ok(Vec::new());
                }

                let mut awards=Vec::with_capacity(mongo_awards.len());
                for mongo_award in mongo_awards.iter() {
                    match *mongo_award {
                        Bson::Document( ref doc ) => {
                            let award:Award = bson::from_bson( bson::Bson::Document(doc.clone()) ).unwrap();//NOTE:clone!

                            awards.push(award);
                        },
                        _ => return Err(Error::Other( format!("Award of user \"{}\" must be struct",self.user_id) )),
                    }
                }

                Ok( awards )
            },
            _ => Err(Error::Other( format!("Awards of user \"{}\" must be array",self.user_id) )),
        }
    }

    pub fn get_avatar(&self) -> Result<ImageID,Error> {
        match self.doc.get("avatar") {
            Some(&Bson::String(ref avatar_s)) => {
                let avatar=Uuid::parse_str(avatar_s.as_str())?;
                Ok(avatar)
            },
            _ => Err(Error::Other( format!("Avatar of user \"{}\" must be string",self.user_id) )),
        }
    }

    pub fn get_threads(&self) -> Result<Vec<ThreadID>,Error> {
        match self.doc.get("threads") {
            Some(&Bson::Array(ref mongo_threads)) => {
                if mongo_threads.len()==0 {
                    return Ok(Vec::new());
                }

                let mut threads=Vec::with_capacity(mongo_threads.len());
                for mongo_thread in mongo_threads.iter() {
                    match *mongo_thread {
                        Bson::String( ref thread_id_s ) => {
                            let thread_id=Uuid::parse_str(thread_id_s.as_str())?;
                            threads.push(thread_id);
                        },
                        _ => return Err(Error::Other( format!("Thread of user \"{}\" must be string",self.user_id) )),
                    }
                }

                Ok( threads )
            },
            _ => Err(Error::Other( format!("Threads of user \"{}\" must be array",self.user_id) )),
        }
    }

}
