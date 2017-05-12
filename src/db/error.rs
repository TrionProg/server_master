use std;
use serde;
use bincode;

use redis;
use cdrs;
use postgres;

pub enum Error{
    IOError(Box<std::io::Error>),
    RedisError(Box<redis::RedisError>),
    CassandraError(Box<cdrs::error::Error>),
    PostgresConnectError(Box<postgres::error::ConnectError>),
    PostgresError(Box<postgres::error::Error>),
    BincodeError(Box<bincode::Error>),
}

impl From<std::io::Error> for Error{
    fn from(io_error:std::io::Error) -> Self{
        Error::IOError( Box::new(io_error) )
    }
}

impl From<redis::RedisError> for Error{
    fn from(redis_error:redis::RedisError) -> Self{
        Error::RedisError( Box::new(redis_error) )
    }
}

impl From<cdrs::error::Error> for Error{
    fn from(cassandra_error:cdrs::error::Error) -> Self{
        Error::CassandraError( Box::new(cassandra_error) )
    }
}

impl From<postgres::error::ConnectError> for Error{
    fn from(postgres_connect_error:postgres::error::ConnectError) -> Self{
        Error::PostgresConnectError( Box::new(postgres_connect_error) )
    }
}

impl From<postgres::error::Error> for Error{
    fn from(postgres_error:postgres::error::Error) -> Self{
        Error::PostgresError( Box::new(postgres_error) )
    }
}

impl From<bincode::Error> for Error{
    fn from(bincode_error:bincode::Error) -> Self{
        Error::BincodeError( Box::new(bincode_error) )
    }
}

impl std::fmt::Display for Error{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self{
            Error::IOError(ref io_error) => write!(f, "IO error: \"{}\"",io_error),
            Error::RedisError(ref redis_error) => write!(f, "Redis error: \"{}\"",redis_error),
            Error::CassandraError(ref cassandra_error) => write!(f, "Cassandra error: \"{}\"",cassandra_error),
            Error::PostgresConnectError(ref postgres_connect_error) => write!(f, "PostgreSQL connect error: \"{}\"",postgres_connect_error),
            Error::PostgresError(ref postgres_error) => write!(f, "PostgreSQL error: \"{}\"",postgres_error),
            Error::BincodeError(ref bincode_error) => write!(f, "Bincode error: \"{}\"",bincode_error),
        }
    }
}
