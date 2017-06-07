

pub fn fill(){
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

    for w in words.iter().take(10) {
        println!("{}",w);
    }

    let mut names:Vec<&str>=Vec::with_capacity(words.len()/10);

    for word in words.iter() {
        if word.chars().next().unwrap().is_uppercase() {
            names.push(word);
        }
    }

    for w in names.iter().take(10) {
        println!("{}",w);
    }

    println!("{} {}",words.len(), names.len());
}
