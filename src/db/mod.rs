
mod error;
pub use self::error::Error;

mod db_clients;
pub use self::db_clients::DBClients;

mod cash;
pub use self::cash::Cash;


mod users;
pub use self::users::{Users,UserShortInformation};
pub use self::users::{AddUserResult,OnlineStatus};

pub type ServerID=i32;
pub type BinaryData=Vec<u8>;
