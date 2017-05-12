use redis;
use cdrs;

use super::Error;

pub struct DBClients{
    pub redis:redis::Client,
    //pub cassandra:cdrs::client::CDRS<cdrs::authenticators::NoneAuthenticator, cdrs::transport::TransportTcp>,
}

impl DBClients {
    pub fn connect() -> Result<Self,Error> {
        let redis_address="redis://127.0.0.1/1";
        let redis_client = redis::Client::open(redis_address)?;

        /*
        let cassandra_address="127.0.0.1:9042";
        let cassandra_transport = cdrs::transport::TransportTcp::new(cassandra_address)?;
        let cassandra_client = cdrs::client::CDRS::new(cassandra_transport, cdrs::authenticators::NoneAuthenticator);
        */

        let db_clients=DBClients {
            redis:redis_client,
            //cassandra:cassandra_client,
        };

        Ok( db_clients )
    }
}
