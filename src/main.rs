#[macro_use]
extern crate serde_derive;

extern crate uuid;
extern crate serde;
extern crate bincode;

extern crate redis;
extern crate cdrs;
extern crate postgres;

pub type BinaryData=Vec<u8>;

mod db;

fn process() -> Result<(),db::Error> {
    let db_clients=db::DBClients::connect()?;
    let mut cash=db::Cash::new(&db_clients)?;

    match cash.get_file("55d16d15-5006-4d90-b682-971f14ac568f")? {
        Some( data ) => println!("Binaty data"),
        None => println!("Not found"),
    }

    let mut users=db::Users::new(&db_clients)?;

    match users.add_user("newbie","455")? {
        db::AddUserResult::Success => println!("success"),
        db::AddUserResult::UserExists => println!("exists"),
    }

    let user_id=users.get_user_id_by_login("newbie")?.unwrap();
    //println!("{:?}",users.get_user_id_by_login("newbie")?);

    let user=users.get_user_short_information_by_id(user_id)?.unwrap();

    println!("{} {}",user.login, user.avatar);

    Ok(())
}

fn main() {
    match process() {
        Ok(_) => {},
        Err(e) => println!("{}",e),
    }
}
