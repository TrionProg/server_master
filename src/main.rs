#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate uuid;
extern crate serde;
extern crate bincode;
extern crate byteorder;

extern crate redis;
extern crate cdrs;
extern crate postgres;
extern crate mongodb;

#[macro_use(bson, doc)]
extern crate bson;

pub type BinaryData=Vec<u8>;

mod db;

fn process() -> Result<(),db::Error> {
    let redis_global_client=db::RedisClient::connect("redis://127.0.0.1/0")?;
    let redis_users_client=db::RedisClient::connect("redis://127.0.0.1/1")?;
    let redis_images_client=db::RedisClient::connect("redis://127.0.0.1/2")?;
    //let redis_threads_watchers_client=db::RedisClient::connect("redis://127.0.0.1/3")?;
    let redis_hot_threads_client=db::RedisClient::connect("redis://127.0.0.1/4")?;
    let mongo_client=db::MongoClient::connect("mongodb://localhost:27017/")?;
    let mondo_users_db=mongo_client.get_db("users");

    //let mut cash=db::Cash::new(&db_clients)?;

    /*
    match cash.get_file("55d16d15-5006-4d90-b682-971f14ac568f")? {
        Some( data ) => println!("Binaty data"),
        None => println!("Not found"),
    }
    */

    let mut global=db::Global::new(&redis_global_client, &mondo_users_db)?;
    global.load()?;
    let mut users=db::Users::new(&redis_users_client, &redis_global_client, &mondo_users_db)?;
    let mut images=db::Images::new(&redis_images_client)?;
    let mut forum=db::Forum::new(&redis_global_client, &redis_hot_threads_client, &mondo_users_db)?;

    match users.add_user("user0","455")? {
        db::AddUserResult::Success(id) => println!("success {}",id),
        db::AddUserResult::UserExists => println!("exists"),
    }

    let user_id=users.get_user_id_by_login("user0")?.unwrap();
    println!("{:?}",users.get_user_id_by_login("newbie")?);

    let user=users.get_short_user_information_by_id(user_id)?.unwrap();
    use std::path::Path;
    //global.set_default_avatars_from_files(&mut images, Path::new("small_avatar.png"), Path::new("big_avatar.png"))?;
    println!("{}",user.login.len());
    println!("{} {}",user.login, user.avatar);

    println!("{} {}",users.user_exists_by_id(user_id)?, users.user_exists_by_id(user_id)?);

    //println!("Added:{}",users.give_award(user_id,"Held des Vaterland","für die Dummheit".to_string())?.is_some());
    //users.smt()?;

    let a=forum.create_first_post(14, chrono::offset::utc::UTC::now(), "hello world".to_string())?;
    println!("{}",a);

    match users.get_full_user_information_by_id(user_id)? {
        Some( full_information ) => {
            println!("awards");
            let awards=full_information.get_awards()?;
            for award in awards.iter() {
                println!("award \"{}\" for \"{}\"",award.name,award.description);
            }
        },
        None => {}
    }

    Ok(())
}

fn main() {
    match process() {
        Ok(_) => {},
        Err(e) => println!("{}",e),
    }
}
