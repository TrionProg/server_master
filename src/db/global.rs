use std;
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
use super::Images;

use postgres::types::ToSql;
use redis::Commands;
use bson::Bson;

pub struct Global{
    redis_global:RedisCollection,
    mongo_global:MongoCollection,
}

impl Global{
    pub fn new(redis_global_client:&RedisClient, mongo_db:&MongoDatabase) -> Result<Self,Error> {
        let redis_global = redis_global_client.get_collection()?;
        let mongo_global = mongo_db.get_collection("global");

        let global=Global{
            redis_global,
            mongo_global,
        };

        Ok( global )
    }

    pub fn create(&self) -> Result<(),Error> {
        let id=Uuid::new(UuidVersion::Random).unwrap();

        let doc = doc! {
            "_id" => 1,
            "default_avatar_small" => "none",
            "default_avatar_big" => "none"
        };

        self.mongo_global.insert_one(doc, None)?;

        Ok(())
    }

    pub fn load(&self) -> Result<(),Error> {
        let find_filter=doc! {
            "_id" => 1
        };

        let doc=self.mongo_global.find_one(Some(find_filter),None)?; //NOTE:None

        let global=match doc {
            Some( doc ) => doc,
            None => return Err(Error::Other( "mongo does not contain \"global\"".to_string() )),
        };


        match global.get("default_avatar_small") {
            Some(&Bson::String(ref s)) => {
                let default_avatar_small=Uuid::parse_str(s.as_str()).unwrap();//NOTE:unwrap
                let default_avatar_small_b:BinaryData=bincode::serialize(&default_avatar_small,bincode::Bounded(96))?;

                self.redis_global.set("default_avatar_small", default_avatar_small_b)?;
            },
            _ => return Err(Error::Other( "\"default_avatar_small\" must be string".to_string() )),
        }

        match global.get("default_avatar_big") {
            Some(&Bson::String(ref s)) => {
                let default_avatar_big=Uuid::parse_str(s.as_str()).unwrap();//NOTE:unwrap
                let default_avatar_big_b:BinaryData=bincode::serialize(&default_avatar_big,bincode::Bounded(96))?;
                
                self.redis_global.set("default_avatar_big", default_avatar_big_b)?;
            },
            _ => return Err(Error::Other( "\"default_avatar_big\" must be string".to_string() )),
        }

        Ok(())
    }

    pub fn set_default_avatars(&self, small_avatar:Uuid, big_avatar:Uuid) -> Result<(),Error> {
        let find_filter=doc! {
            "_id" => 1
        };

        let default_avatar_small_s=small_avatar.to_string();
        let default_avatar_big_s=big_avatar.to_string();

        let update_doc=doc! {
            "$set" => {
                "default_avatar_small" => default_avatar_small_s,
                "default_avatar_big" => default_avatar_big_s
            }
        };

        self.mongo_global.find_one_and_update( find_filter ,update_doc, None );//TODO:None

        let default_avatar_small_b:BinaryData=bincode::serialize(&small_avatar,bincode::Bounded(96))?;
        let default_avatar_big_b:BinaryData=bincode::serialize(&big_avatar,bincode::Bounded(96))?;

        self.redis_global.set("default_avatar_small", default_avatar_small_b)?;
        self.redis_global.set("default_avatar_big", default_avatar_big_b)?;

        Ok(())
    }

    pub fn set_default_avatars_from_files(&self,
        images:&mut Images,
        path_to_small_avatar:&std::path::Path,
        path_to_big_avatar:&std::path::Path
    ) -> Result<(),Error> {
        let small_avatar_id=images.add_from_file(1, path_to_small_avatar)?;
        let big_avatar_id=images.add_from_file(1, path_to_big_avatar)?;

        self.set_default_avatars(small_avatar_id, big_avatar_id)
    }
}
