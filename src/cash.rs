use redis;
use cdrs;

use super::Error;
use super::DBClients;
use super::BinaryData;

const FILE_EXPIRATION:usize=30;

pub struct Cash {
    redis_connection:redis::Connection,
    cassandra_session:cdrs::client::Session<cdrs::authenticators::NoneAuthenticator, cdrs::transport::TransportTcp>,
}

impl Cash {
    pub fn new(db_clients:&DBClients) -> Result<Self,Error> {
        let redis_connection = db_clients.redis.get_connection()?;

        let cassandra_address="127.0.0.1:9042";
        let cassandra_transport = cdrs::transport::TransportTcp::new(cassandra_address)?;
        let cassandra_client = cdrs::client::CDRS::new(cassandra_transport, cdrs::authenticators::NoneAuthenticator);
        let mut cassandra_session = cassandra_client.start(cdrs::compression::Compression::None)?;

        cassandra_session.query(
            cdrs::query::QueryBuilder::new("USE files;").finalize(),
            true,
            true
        )?;

        let cash=Cash{
            redis_connection:redis_connection,
            cassandra_session:cassandra_session,
        };

        Ok(cash)
    }

    pub fn get_file(&mut self, key:&str) -> Result<Option<BinaryData>,Error> {
        use redis::Commands;
        use cdrs::query::QueryBuilder;
        use cdrs::types::IntoRustByName;

        let data:Option<BinaryData> = self.redis_connection.get(key)?;

        match data {
            Some( data ) => {
                self.redis_connection.expire(key, FILE_EXPIRATION)?;
                return Ok(Some(data));
            },
            None => {},
        }

        let cassandra_querry=QueryBuilder::new(format!("SELECT content from files WHERE id = {};",key)).finalize();

        let cassandra_result=self.cassandra_session.query(cassandra_querry, false, false)?;
        let result_body = cassandra_result.get_body()?;
        let rows=result_body.into_rows().unwrap();

        if rows.len()==0 {
            return Ok(None);
        }

        let content:BinaryData=rows[0].get_by_name("content")?.unwrap();

        self.redis_connection.set_ex(key, content.clone(), FILE_EXPIRATION)?; //NOTE:Clone.. schreck.

        Ok( Some(content) )
    }
}
