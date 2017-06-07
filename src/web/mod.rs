
use std::thread;
use std::sync::{Mutex,RwLock,Arc,Barrier,Weak};

use iron::prelude::*;
use iron::status;
use router::Router;

use iron::error::HttpError;
use iron::Listening;
use iron::mime::Mime;

use db::{Global,Users,Images,Forum,ImageID};

pub struct WebInterface{
    global:Mutex<Global>,
    users:Mutex<Users>,
    images:Mutex<Images>,
    forum:Mutex<Forum>,

    mimeTypes:MimeTypes,
    listener:Mutex<Option<Listening>>,
}

struct MimeTypes{
    text:Mime,
    html:Mime,
    png:Mime,
}

impl WebInterface {
    pub fn run( global:Global, users:Users, images:Images, forum:Forum ) -> Result<Arc<WebInterface>, String> {
        let mimeTypes=MimeTypes{
            text:"text/plane".parse::<Mime>().unwrap(),
            html:"text/html".parse::<Mime>().unwrap(),
            png: "img/png".parse::<Mime>().unwrap(),
        };

        let webInterface=Arc::new(WebInterface{
            global:Mutex::new(global),
            users:Mutex::new(users),
            images:Mutex::new(images),
            forum:Mutex::new(forum),

            mimeTypes:mimeTypes,
            listener:Mutex::new(None),

        });

        match WebInterface::runHTTPListener( webInterface.clone() ){
            Ok ( _ ) => Ok(webInterface),
            Err( e ) => Err(format!("Can not create HTTP listener : {}",e)),
        }
    }

    fn runHTTPListener( webInterface:Arc< WebInterface > ) -> Result<(), HttpError>  {
        let mut router = Router::new();

        /*
        let router_webInterface=webInterface.clone();
        router.get("/", move |r: &mut Request| Ok(Response::with(
            (status::Ok, (*router_webInterface.files.index.read().unwrap()).clone())
        )) );
        */

        let router_webInterface=webInterface.clone();
        router.get("/", move |r: &mut Request| Ok(Response::with((
            router_webInterface.mimeTypes.html.clone(),
            status::Ok,
            format!("hahah"),
        ))), "index" );

        let router_webInterface=webInterface.clone();
        router.get("/threads", move |r: &mut Request| Ok(Response::with((
            router_webInterface.mimeTypes.html.clone(),
            status::Ok,
            WebInterface::show_threads(r,&router_webInterface),
        ))), "threads" );

        let router_webInterface=webInterface.clone();
        router.get("/thread", move |r: &mut Request| Ok(Response::with((
            router_webInterface.mimeTypes.html.clone(),
            status::Ok,
            WebInterface::show_thread(r,&router_webInterface),
        ))), "thread" );

        let router_webInterface=webInterface.clone();
        router.get("/image", move |r: &mut Request| Ok(Response::with((
            router_webInterface.mimeTypes.png.clone(),
            status::Ok,
            WebInterface::send_image(r,&router_webInterface),
        ))), "image" );
        /*
        let router_webInterface=webInterface.clone();
        router.get("/login", move |r: &mut Request| Ok(Response::with((router_webInterface.mimeTypes.text.clone(),
            status::Ok, WebInterface::contentFromFile("Files/loginAnswer.txt"))
        )) );

        let router_webInterface=webInterface.clone();
        router.get("/crypto", move |r: &mut Request| Ok(Response::with((router_webInterface.mimeTypes.html.clone(),
            status::Ok, WebInterface::contentFromFile("Files/Crypto/index.html"))
        )) );

        let router_webInterface=webInterface.clone();
        router.get("/sodium.js", move |r: &mut Request| Ok(Response::with((router_webInterface.mimeTypes.text.clone(),
            status::Ok, WebInterface::contentFromFile("Files/Crypto/sodium.js"))
        )) );

        let router_webInterface=webInterface.clone();
        router.post("/login", move |r: &mut Request|
            match WebInterface::login(r,&router_webInterface) {
                Ok ( msg ) =>
                    Ok(Response::with( (router_webInterface.mimeTypes.text.clone(), status::Ok, msg) )),
                Err( msg ) => {
                    router_webInterface.appData.upgrade().unwrap().log.write(msg);
                    Ok(Response::with( (router_webInterface.mimeTypes.text.clone(), status::BadRequest, String::from(msg)) ))
                }
            }
        );

        let router_webInterface=webInterface.clone();
        router.post("/arenews", move |r: &mut Request|
            match WebInterface::checkNews(r,&router_webInterface) {
                Ok ( responseCipherBase64 ) =>
                    Ok(Response::with( (router_webInterface.mimeTypes.text.clone(), status::Ok, responseCipherBase64) )),
                Err( msg ) => {
                    router_webInterface.appData.upgrade().unwrap().log.write(msg);
                    Ok(Response::with( (router_webInterface.mimeTypes.text.clone(), status::BadRequest, String::from(msg)) ))
                }
            }
        );

        let router_webInterface=webInterface.clone();
        router.post("/cmd", move |r: &mut Request|
            match WebInterface::processCommands(r,&router_webInterface) {
                Ok ( msg ) =>
                    Ok(Response::with( (router_webInterface.mimeTypes.text.clone(), status::Ok, msg) )),
                Err( msg ) => {
                    router_webInterface.appData.upgrade().unwrap().log.write(msg);
                    Ok(Response::with( (router_webInterface.mimeTypes.text.clone(), status::BadRequest, String::from(msg)) ))
                }
            }
        );

        let router_webInterface=webInterface.clone();
        router.post("/logout", move |r: &mut Request|
            match WebInterface::logout(r,&router_webInterface) {
                Ok ( _ ) =>
                    Ok(Response::with( (router_webInterface.mimeTypes.text.clone(), status::Ok, String::from("")) )),
                Err( msg ) => {
                    router_webInterface.appData.upgrade().unwrap().log.write(msg);
                    Ok(Response::with( (router_webInterface.mimeTypes.text.clone(), status::BadRequest, String::from(msg)) ))
                }
            }
        );
        */

        let listener=try!(Iron::new(router).http("0.0.0.0:8080"));

        *webInterface.listener.lock().unwrap()=Some(listener);

        println!("[INFO]Web interface is ready");

        Ok( () )
    }

    pub fn show_threads(req: &mut Request, wi:&Arc< WebInterface >) -> String {
        use urlencoded::UrlEncodedQuery;

        let url_args=match req.get_ref::<UrlEncodedQuery>() {
            Ok(url_args) => url_args,
            Err(e) => return "url error".to_string(),
        };

        let category=match url_args.get("category"){
            Some( cat_str ) => cat_str[0].parse::<i32>().unwrap(),
            None => 0,
        };

        let threads=match wi.forum.lock().unwrap().get_threads(category){
            Ok(threads) => threads,
            Err(e) => return format!("Get threads Error:{}",e),
        };

        let mut out=String::with_capacity(threads.len()*128);
        out.push_str("<!DOCTYPE HTML><HTML lang=\"en\"><HEAD><meta charset='utf-8'></HEAD><BODY><table  cellpadding=\"10\" border=\"1\" width=\"800px\">");

        for thread in threads {
            let short_user_info=match wi.users.lock().unwrap().get_short_user_information_by_id(thread.author){
                Ok(found_short_user_info) => match found_short_user_info {
                    Some(short_user_info) => short_user_info,
                    None => return format!("No user {}",thread.author),
                },
                Err(e) => return format!("Get user Error:{}",e),
            };

            let thread_html=format!("
            <tr>
                <td><img src=\"/image?id={}\" alt=\"{}\"></td>
                <td><a href=\"/user?id={}\">{}</a></td>
                <td><a href=\"/thread?id={}\">{}</a></td>
            </tr>",
                short_user_info.avatar,short_user_info.login,
                thread.author,short_user_info.login,
                thread.id,thread.caption
            );
            out.push_str(&thread_html);
        }

        out.push_str("</table></BODY></HTML>");
        out
    }

    pub fn send_image(req: &mut Request, wi:&Arc< WebInterface >) -> String {
        use urlencoded::UrlEncodedQuery;

        let url_args=match req.get_ref::<UrlEncodedQuery>() {
            Ok(url_args) => url_args,
            Err(e) => return "url error".to_string(),
        };

        let id=match url_args.get("id"){
            Some( id_str ) => match ImageID::parse_str(id_str[0].as_str()) {
                Ok(id) => id,
                Err(_) => return format!("Can not parse id \"{}\"",id_str[0]),
            },
            None => return "no id".to_string(),
        };

        let image_data=match wi.images.lock().unwrap().get_image_data(id){
            Ok(found_image_data) => match found_image_data {
                Some(image_data) => image_data,
                None => return format!("No image \"{}\"",id),
            },
            Err(e) => return format!("Get threads Error:{}",e),
        };

        unsafe{
            String::from_utf8_unchecked(image_data)
        }
    }

    pub fn show_thread(req: &mut Request, wi:&Arc< WebInterface >) -> String {
        use urlencoded::UrlEncodedQuery;

        let url_args=match req.get_ref::<UrlEncodedQuery>() {
            Ok(url_args) => url_args,
            Err(e) => return "url error".to_string(),
        };

        let id=match url_args.get("id"){
            Some( id_str ) => match ImageID::parse_str(id_str[0].as_str()) {
                Ok(id) => id,
                Err(_) => return format!("Can not parse id \"{}\"",id_str[0]),
            },
            None => return "no id".to_string(),
        };

        let thread_post_ids=match wi.forum.lock().unwrap().get_all_post_ids_for_thread(id){
            Ok(thread_post_ids) => thread_post_ids,
            Err(e) => return format!("Get posts Error:{}",e),
        };

        let mut out=String::with_capacity(thread_post_ids.len()*512);
        out.push_str("<!DOCTYPE HTML><HTML lang=\"en\"><HEAD><meta charset='utf-8'></HEAD><BODY><table  cellpadding=\"10\" border=\"1\" width=\"800px\">");

        for post_id in thread_post_ids {
            let post=match wi.forum.lock().unwrap().get_post(post_id) {
                Ok(found_post) => match found_post {
                    Some(post) => post,
                    None => return format!("No post {}",post_id),
                },
                Err(e) => return format!("Get post Error:{}",e),
            };

            let short_user_info=match wi.users.lock().unwrap().get_short_user_information_by_id(post.author){
                Ok(found_short_user_info) => match found_short_user_info {
                    Some(short_user_info) => short_user_info,
                    None => return format!("No user {}",post.author),
                },
                Err(e) => return format!("Get user Error:{}",e),
            };

            let post_html=format!("
            <tr>
                <td><img src=\"/image?id={}\" alt=\"{}\"></td>
                <td><a href=\"/user?id={}\">{}</a><br>Rating:{}</td>
            </tr>
            <tr><td>{}<td></tr>",
                short_user_info.avatar, short_user_info.login,
                post.author, short_user_info.login, short_user_info.rating,
                post.message
            );

            out.push_str(&post_html);
        }

        out.push_str("</table></BODY></HTML>");
        out
    }

    pub fn close(&self){
        println!("[INFO]Closing web interface");

        match *self.listener.lock().unwrap() {
            Some( ref mut listener ) => {listener.close();},
            None => {},
        }

        println!("[INFO]Web interface has been closed");
    }
}
