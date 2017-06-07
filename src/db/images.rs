use std;
use bincode;

use redis;
use cdrs;
use postgres;
use mongodb;

use bson;

use uuid::{Uuid,UuidVersion};

use super::Error;
use super::{BinaryData,ServerID,UserID,ImageID};
use super::{RedisClient,MongoDatabase};
use super::{RedisCollection,MongoCollection,CassandraSession,PostgresSession};

use postgres::types::ToSql;
use redis::Commands;
use bson::Bson;
use cdrs::query::QueryBuilder;
use cdrs::types::IntoRustByName;

const IMAGE_EXPIRATION:usize=60;

pub struct Images{
    redis_images:RedisCollection,
    postgres_session:PostgresSession,
}

impl Images {
    pub fn new(redis_images_client:&RedisClient) -> Result<Self,Error> {
        let redis_images = redis_images_client.get_collection()?;
        let postgres_session = Self::connect_to_postgres()?;

        let images=Images{
            redis_images,
            postgres_session,
        };

        Ok( images )
    }

    fn connect_to_postgres() -> Result<PostgresSession,Error> {
        let tls_mode = postgres::TlsMode::None;
        let postgres_session = PostgresSession::connect("postgresql://postgresql_user:user@localhost/users",tls_mode)?;

        Ok( postgres_session )
    }

    pub fn add_image(&mut self, author:UserID, data:BinaryData) -> Result<ImageID,Error> {
        loop{
            let id=Uuid::new(UuidVersion::Random).unwrap();

            let insert_result=self.postgres_session.execute(
                "INSERT INTO images (id, author, date, data) VALUES($1, $2, current_date, $3)",
                &[&id,&author,&data]
            )?;

            if insert_result==1 {
                return Ok(id);
            }
        }
    }

    pub fn add_from_file(&mut self, author:UserID, path_to_file:&std::path::Path) -> Result<ImageID,Error> {
        use std::fs::File;
        use std::io::BufReader;
        use std::io::prelude::*;

        let file = File::open(path_to_file)?;
        let mut buf_reader = BufReader::new(file);
        let mut content = Vec::new();
        buf_reader.read_to_end(&mut content)?;

        self.add_image(author, content)
    }

    pub fn get_image_data(&mut self, id:ImageID) -> Result<Option<BinaryData>,Error> {
        let data:Option<BinaryData> = self.redis_images.get( id.to_string() )?;

        match data {
            Some( data ) => {
                return Ok(Some(data));
            },
            None => {},
        }

        let result_rows=self.postgres_session.query(
            "SELECT data FROM images WHERE id=$1",
            &[&id]
        )?;

        if result_rows.len()>0 {
            let data:BinaryData=result_rows.get(0).get(0);
            self.redis_images.set_ex(id.to_string(), data.clone(), IMAGE_EXPIRATION)?; //NOTE:Clone.. schreck.

            Ok( Some(data) )
        }else{
            Ok( None )
        }
    }

    pub fn remove_image(&mut self, id:ImageID) -> Result<bool,Error> {
        let remove_result=self.postgres_session.execute(
            "DELETE FROM images WHERE id = $1;",
            &[&id]
        )?;

        if remove_result!=1 {
            return Ok(false);
        }

        self.redis_images.del(id.to_string())?;

        Ok(true)
    }
}
