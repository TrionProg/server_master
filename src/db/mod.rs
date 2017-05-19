
mod error;
pub use self::error::Error;

mod db_clients;
pub use self::db_clients::{RedisClient,MongoClient,MongoDatabase};

mod global;
pub use self::global::{Global};

//mod cash;
//pub use self::cash::Cash;

mod users;
pub use self::users::{Users,ShortUserInformation,FullUserInformation};
pub use self::users::{AddUserResult,OnlineStatus};

mod images;
pub use self::images::{Images};

mod forum;
pub use self::forum::{Forum,Category};

pub type ServerID=i32;
pub type BinaryData=Vec<u8>;
pub type UserID=i32;

use redis;
use mongodb;
use cdrs;
use postgres;

pub type RedisCollection=redis::Connection;
pub type MongoCollection=mongodb::coll::Collection;
pub type CassandraSession=cdrs::client::Session<cdrs::authenticators::NoneAuthenticator, cdrs::transport::TransportTcp>;
pub type PostgresSession=postgres::Connection;
