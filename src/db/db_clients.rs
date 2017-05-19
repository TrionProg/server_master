
use redis;
use cdrs;
use mongodb;

use super::Error;

pub struct RedisClient{
    client:redis::Client,
}

impl RedisClient {
    pub fn connect(redis_address:&str) -> Result<Self,Error> {
        let client=redis::Client::open(redis_address)?;

        Ok( RedisClient{client} )
    }

    pub fn get_collection(&self) -> Result<super::RedisCollection,Error> {
        let redis_connection = self.client.get_connection()?;

        Ok( redis_connection )
    }
}

pub struct MongoClient{
    client:mongodb::Client,
}

impl MongoClient {
    pub fn connect(mongo_address:&str) -> Result<Self,Error> {
        use mongodb::ThreadedClient;
        let client=mongodb::Client::with_uri(mongo_address)?;

        Ok( MongoClient{client} )
    }

    pub fn get_db(&self, db_name:&str) -> MongoDatabase {
        use mongodb::ThreadedClient;
        let db=self.client.db(db_name);

        MongoDatabase::new(db)
    }
}

pub struct MongoDatabase{
    db:mongodb::db::Database
}

impl MongoDatabase {
    pub fn new(db:mongodb::db::Database) -> Self {
        MongoDatabase{
            db:db
        }
    }

    pub fn get_collection(&self, collection_name:&str) -> super::MongoCollection {
        use mongodb::db::ThreadedDatabase;
        let collection=self.db.collection(collection_name);

        collection
    }
}
