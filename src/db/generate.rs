use db;
use db::{Global,Users,Images,Forum,ImageID,Error};
use rand;
use rand::distributions::{IndependentSample, Range};
use chrono::UTC;

pub fn fill(global:&mut Global, users:&mut Users, images:&mut Images, forum:&mut Forum) -> Result<(),Error>{
    use std::fs::File;
    use std::io::BufReader;
    use std::io::prelude::*;

    println!("reading zaratustra");

    let file = File::open("zaratustra.txt").unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut content = String::with_capacity(120000);
    buf_reader.read_to_string(&mut content).unwrap();

    println!("parsing zaratustra");

    let words:Vec<&str>=content.split_whitespace().filter(|w| {
        if w.len()==0 {
            return false;
        }

        for c in w.chars() {
            if !c.is_alphabetic() {
                return false;
            }
        }

        true
    }).collect();

    let mut names:Vec<&str>=Vec::with_capacity(words.len()/10);

    for word in words.iter() {
        if word.chars().next().unwrap().is_uppercase() {
            names.push(word);
        }
    }


    const USERS_COUNT:usize = 10;
    const CATEGORIES_COUNT:usize = 10;
    const THREADS_COUNT:usize = 10;
    const POSTS_COUNT:usize = 10;

    let mut rng = rand::thread_rng();
    /*
    println!("Clearing users");
    users.clear()?;
    println!("Generating users");

    for i in 0..USERS_COUNT {
        loop {
            let name=names[Range::new(0,names.len()-1).ind_sample(&mut rng)];
            let password=words[Range::new(0,words.len()-1).ind_sample(&mut rng)];

            let user_id=match users.add_user(name,password)? {
                db::AddUserResult::Success(id) => id,
                db::AddUserResult::UserExists => continue,
            };

            break;
        }
    }
    */

    let user_ids=users.get_user_ids()?;
    /*
    println!("Clearing forum");
    forum.clear()?;
    println!("Generating forum");

    for category in 0..CATEGORIES_COUNT {
        for _ in 0..THREADS_COUNT {
            let author = user_ids[Range::new(0,user_ids.len()-1).ind_sample(&mut rng)];
            let thread_caption=gen_text(&words, 6);
            let first_post_message=gen_text(&words, Range::new(50,200).ind_sample(&mut rng));

            let forum_id=forum.create_thread(&users, author, category as i32, thread_caption, first_post_message)?;

            for _ in 0..POSTS_COUNT {
                let author = user_ids[Range::new(0,user_ids.len()-1).ind_sample(&mut rng)];
                let message=gen_text(&words, Range::new(50,200).ind_sample(&mut rng));
                forum.add_post(forum_id, author, UTC::now(), message)?;
            }
        }
    }
    */

    Ok(())
}

fn gen_text(words:&Vec<&str>, words_count:usize) -> String {
    let mut text=String::with_capacity(words_count*8);
    let mut rng = rand::thread_rng();

    let first_word=Range::new(0,words.len()-words_count-1).ind_sample(&mut rng);
    for i in first_word..first_word+words_count{//words.iter().skip(first_word).take(words_count) {
        text.push_str(words[i]);
        text.push(' ');
    }

    /*
    for _ in 0..words_count {
        let word=words[Range::new(0,words.len()-1).ind_sample(&mut rng)];
        text.push_str(word);
        text.push(' ');
    }
    */

    text
}
