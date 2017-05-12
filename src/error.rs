use std;
use redis;
use cdrs;

pub enum Error{
    IOError(Box<std::io::Error>),
    RedisError(Box<redis::RedisError>),
    CassandraError(Box<cdrs::error::Error>),
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

impl std::fmt::Display for Error{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self{
            Error::IOError(ref io_error) => write!(f, "IO error: \"{}\"",io_error),
            Error::RedisError(ref redis_error) => write!(f, "Redis error: \"{}\"",redis_error),
            Error::CassandraError(ref cassandra_error) => write!(f, "Cassandra error: \"{}\"",cassandra_error),
        }
    }
}
