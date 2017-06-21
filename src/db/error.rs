use std;
use serde;
use bincode;

use redis;
use cdrs;
use postgres;
use mongodb;
use bson;
use uuid;
use rusted_cypher;

pub enum Error{
    IOError(Box<std::io::Error>),
    RedisError(Box<redis::RedisError>),
    CassandraConnectionError(Box<std::io::Error>),
    CassandraError(Box<cdrs::error::Error>),
    PostgresConnectError(Box<postgres::error::ConnectError>),
    PostgresError(Box<postgres::error::Error>),
    BincodeError(Box<bincode::Error>),
    MongoDBError(Box<mongodb::error::Error>),
    BsonIncodeError(Box<bson::EncoderError>),
    BsonDecodeError(Box<bson::DecoderError>),
    UuidParseError(Box<uuid::ParseError>),
    Neo4jError(Box<rusted_cypher::GraphError>),
    Other(String),
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

impl From<mongodb::error::Error> for Error {
    fn from(mongodb_error:mongodb::error::Error) -> Self{
        Error::MongoDBError( Box::new(mongodb_error) )
    }
}

impl From<bson::EncoderError> for Error {
    fn from(bson_incode_error:bson::EncoderError) -> Self{
        Error::BsonIncodeError( Box::new(bson_incode_error) )
    }
}

impl From<bson::DecoderError> for Error {
    fn from(bson_decode_error:bson::DecoderError) -> Self{
        Error::BsonDecodeError( Box::new(bson_decode_error) )
    }
}

impl From<uuid::ParseError> for Error {
    fn from(uuid_parse_error:uuid::ParseError) -> Self{
        Error::UuidParseError( Box::new(uuid_parse_error) )
    }
}

impl From<rusted_cypher::GraphError> for Error {
    fn from(neo4j_error:rusted_cypher::GraphError) -> Self{
        Error::Neo4jError( Box::new(neo4j_error) )
    }
}

impl std::fmt::Display for Error{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self{
            Error::IOError(ref io_error) => write!(f, "IO error: \"{}\"",io_error),
            Error::RedisError(ref redis_error) => write!(f, "Redis error: \"{}\"",redis_error),
            Error::CassandraConnectionError(ref cassandra_connect_error) => write!(f, "Cassandra connect error: \"{}\"",cassandra_connect_error),
            Error::CassandraError(ref cassandra_error) => write!(f, "Cassandra error: \"{}\"",cassandra_error),
            Error::PostgresConnectError(ref postgres_connect_error) => write!(f, "PostgreSQL connect error: \"{}\"",postgres_connect_error),
            Error::PostgresError(ref postgres_error) => write!(f, "PostgreSQL error: \"{}\"",postgres_error),
            Error::BincodeError(ref bincode_error) => write!(f, "Bincode error: \"{}\"",bincode_error),
            Error::MongoDBError(ref mongodb_error) => write!(f, "MongoDB error: \"{}\"",mongodb_error),
            Error::BsonIncodeError(ref bson_incode_error) => write!(f, "BSON Encode Error: \"{}\"",bson_incode_error),
            Error::BsonDecodeError(ref bson_decode_error) => write!(f, "BSON Decode Error: \"{}\"",bson_decode_error),
            Error::UuidParseError(ref uuid_parse_error) => write!(f, "UUID parse Error: \"{}\"",uuid_parse_error),
            Error::Neo4jError(ref neo4j_error) => {
                write!(f, "Neo4j Error: \"{:?}\"",neo4j_error)?;

                let b:&rusted_cypher::GraphError=&neo4j_error;

                match *b {
                    rusted_cypher::GraphError::Neo4j(ref errors) => {
                        for e in errors.iter() {
                            write!(f, "\"{:?}\"",e)?;
                        }
                    }
                    _ => {},
                }

                Ok(())
            },
            Error::Other(ref msg) => write!(f, "{}",msg),
        }
    }
}
