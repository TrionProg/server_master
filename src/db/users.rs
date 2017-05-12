use bincode;

use redis;
use cdrs;
use postgres;

use uuid::Uuid;

use super::Error;
use super::DBClients;
use super::{BinaryData,ServerID};

type CassandraSession=cdrs::client::Session<cdrs::authenticators::NoneAuthenticator, cdrs::transport::TransportTcp>;
pub type UserID=i32;

const FILE_EXPIRATION:usize=30;

use serde::Serialize;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserShortInformation{
    pub login:String,
    pub avatar:Uuid,
    pub rating:f32,
    pub online_status:OnlineStatus,
}

pub struct Users{
    redis_connection:redis::Connection,
    cassandra_session:CassandraSession,
    postgresql_connection:postgres::Connection,
}

pub enum AddUserResult{
    UserExists,
    Success,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OnlineStatus{
    Offline,
    Online,
    Playing(ServerID),
}


impl Users {
    pub fn new(db_clients:&DBClients) -> Result<Self,Error> {
        let redis_connection = Self::connect_to_redis(db_clients)?;
        let cassandra_session = Self::connect_to_cassandra(db_clients)?;
        let postgresql_connection = Self::connect_to_postgresql(db_clients)?;

        let users=Users{
            redis_connection,
            cassandra_session,
            postgresql_connection,
        };

        Ok( users )
    }

    fn connect_to_redis(db_clients:&DBClients) -> Result<redis::Connection,Error> {
        let redis_connection= db_clients.redis.get_connection()?;

        Ok( redis_connection )
    }

    fn connect_to_cassandra(db_clients:&DBClients) -> Result<CassandraSession,Error> {
        let cassandra_address="127.0.0.1:9042";
        let cassandra_transport = cdrs::transport::TransportTcp::new(cassandra_address)?;
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

    fn connect_to_postgresql(db_clients:&DBClients) -> Result<postgres::Connection,Error> {
        let tls_mode = postgres::TlsMode::None;
        let postgresql_connection = postgres::Connection::connect("postgresql://postgresql_user:user@localhost/users",tls_mode)?;

        Ok( postgresql_connection )
    }

    pub fn add_user(&self, login:&str, password:&str) -> Result<AddUserResult,Error> {
        use postgres::types::ToSql;
        let exists_result_rows=self.postgresql_connection.query(
            "SELECT EXISTS(SELECT 1 FROM users WHERE login=$1)",
            &[&login]
        )?;

        if exists_result_rows.len()>0 {
            let exists:bool=exists_result_rows.get(0).get(0);

            if exists {
                return Ok(AddUserResult::UserExists);
            }
        }

        let insert_result=self.postgresql_connection.execute(
            "INSERT INTO users (login,password,avatar,rating) VALUES ($1, $2, NULL, 0.0)",
            &[&login,&password]
        )?;

        if insert_result!=1 {
            return Ok(AddUserResult::UserExists);
        }

        Ok( AddUserResult::Success )
    }

    pub fn get_user_id_by_login(&self, login:&str) -> Result<Option<UserID>, Error> {
        let result_rows=self.postgresql_connection.query(
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

    pub fn get_user_short_information_by_id(&self, user_id:UserID) -> Result<Option<UserShortInformation>,Error> {
        use redis::Commands;

        let data:Option<BinaryData> = self.redis_connection.get(user_id)?;

        match data {
            Some( data ) => {
                let user:UserShortInformation=bincode::deserialize(&data)?;
                return Ok( Some(user) );
            },
            None => {},
        }

        let result_rows=self.postgresql_connection.query(
            "SELECT login,avatar,rating FROM users WHERE id=$1",
            &[&user_id]
        )?;

        println!("--");

        if result_rows.len()>0 {
            let login:String=result_rows.get(0).get(0);
            let avatar:Uuid=result_rows.get(0).get(1);
            let rating:f32=result_rows.get(0).get(2);
            let online_status=OnlineStatus::Offline;

            let user=UserShortInformation{
                login,
                avatar,
                rating,
                online_status
            };

            let data:BinaryData=bincode::serialize(&user,bincode::Bounded(96))?;
            self.redis_connection.set_ex(user_id, data, FILE_EXPIRATION)?;

            Ok( Some(user) )
        }else{
            Ok( None )
        }
    }

}
