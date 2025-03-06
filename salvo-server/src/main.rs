use std::ffi::OsStr;
use std::path::Path;
use std::{fs, io};
use std::io::{copy, Read, Seek, Write};
use std::str;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key // Or `Aes128Gcm`
};
use time::UtcOffset;
use tracing::level_filters::LevelFilter;
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, Layer};
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use zip::write::FileOptions;
use chrono::{DateTime, FixedOffset, Utc};
use salvo::http::cookie::time::macros;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use walkdir::{DirEntry, WalkDir};
use salvo::{fs::NamedFile, prelude::*};
use zip::ZipWriter;
use tokio_cron_scheduler::Job;
use windows_service::{
    service::ServiceAccess,
    service_manager::{ServiceManager, ServiceManagerAccess},
};

const FILE_CS: &str = ".zip.crabns";

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let mut rust = std::env::var("rustserver").expect("请配置环境变量rustserver到指定文件夹");
    log_init(rust).await;
    
    encrypted_id::init("23t4y567kuydw3456ukjhgfd8*&&%￥#");
    let mut sched = tokio_cron_scheduler::JobScheduler::new().await.unwrap();
    // 非东八区时间
    sched.add(
        Job::new_async("0 0 0 * * *", |_uuid, _l| {
            Box::pin(async move {
                let utc_datetime: DateTime<Utc> = Utc::now();
                let rust = std::env::var("rustserver").expect("请配置环境变量rustserver到指定文件夹");
                let offset = FixedOffset::east(8 * 3600); // 8小时偏移量

                // 将UTC时间转换为东八区时间
                let beijing_time: DateTime<FixedOffset> = utc_datetime.with_timezone(&offset);
                println!("{beijing_time:?}");
                let mut utc  = beijing_time.format("%Y-%m-%d").to_string();
                encrypt_file(&rust, utc).await;
            })
        }).unwrap()
    ).await.unwrap();
    sched.start().await;
    let router = Router::with_path("download").post(send_file);
    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
    Server::new(acceptor).serve(router).await;
}

#[handler]
async fn intercept_all(req: &mut Request, res: &mut Response, depot: &mut Depot) -> Result<(), StatusError> {
    let path = req.uri().path();
    
    if path != "/download" {
        res.render(Text::Plain("Access Denied".to_string()));
        res.status_code(StatusCode::FORBIDDEN);
        return Err(StatusError::from_code(StatusCode::FORBIDDEN).unwrap());
    }
    
    Ok(())
}



#[handler]
async fn send_file(req: &mut Request, res: &mut Response) {
    let body = req.payload().await.unwrap();

    let vec = body.to_vec();
    let s = String::from_utf8(vec).unwrap();
    println!("{s}");
    match  encrypted_id::decrypt(&s, ":>><<::{3rfsaqwwkeyddadadaw1w*%@^%&@($(@)$"){
        Err(e) => {
            // res.headers_mut().insert("Content-Type", "text/html".parse().unwrap());
            // res.write_body("<h1>无权访问</h1>").unwrap();
            res.headers_mut().insert("Content-Type", "text/plain; charset=utf-8".parse().unwrap());
            res.write_body("无权访问").unwrap();
        }, 
        Ok(fl) => {
            match req.remote_addr().clone().into_std() {
                Some(ip) => {
                    let ip = ip.ip().to_string() + "访问成功开始传输....";
                    tracing::info!(ip); 
                    // let fil = fl.to_string().to_string();
                    let time = chrono::DateTime::from_timestamp(fl as i64, 0).unwrap();
                    let mut utc  = time.format("%Y-%m-%d").to_string();
                    let mut rustserver = std::env::var("rustserver").unwrap();
                    let path = std::env::var("rustserver").unwrap();
                    println!("{path}");
                    rustserver.push_str(&utc);
                    rustserver.push_str(FILE_CS);
                    let pathfile = std::path::Path::new(&rustserver);
                    
                    if pathfile.exists() {
                        NamedFile::open(rustserver).await.unwrap().send(req.headers(), res).await;
                    } else {
                        let en_file = encrypt_file(&path, utc).await;
                        NamedFile::open(en_file).await.unwrap().send(req.headers(), res).await;
                    }
                },
                None => {
                    res.headers_mut().insert("Content-Type", "text/plain; charset=utf-8".parse().unwrap());
                    res.write_body("错误  请重新尝试").unwrap();
                    // res.headers_mut().insert("Content-Type", "text/html".parse().unwrap());
                    // res.write_body("<h1>错误  请重新尝试</h1>").unwrap();
                }
            }
        }
    }
    // let rustserver = std::env::var("rustserver").unwrap();
    // let utc_datetime: chrono::DateTime<Utc> = Utc::now();
    // let mut utc  = utc_datetime.format("%Y-%m-%d").to_string();
    // let en_file = encrypt_file("D:\\Program Files (x86)\\axum-sqlite", utc).await;
    // 压缩两个文件
    // compress_dir(Path::new("d:/tmp"), Path::new("d:/tmp.zip"));
    // let a = NamedFile::open("./axum-main111.zip").await.unwrap();
    // let key = Key::<Aes256Gcm>::from_slice(b"key$&%Q@KKkaq");
    // let cipher = Aes256Gcm::new(&key);
    // let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
    // let ciphertext = cipher.encrypt(&nonce, b"plaintext message".as_ref()).unwrap();
    // let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref()).unwrap();
    // req.headers_mut().insert("key", "valdaw".parse().unwrap());
    // a.send(req.headers(), res).await;
    // 生成密钥和nonce
    // let connt = "attachment; filename=".to_string();
    // connt.push_str(&en_file);
    
    // // 设置响应头并返回加密后的文件内容
    // res.headers_mut().insert("Content-Disposition", connt.parse().unwrap());
    // res.write_body(encrypted_data).unwrap();
    
    
    // let mut path = "./".to_string();
    // path.push_str(&en_file);

    // NamedFile::open(path).await.unwrap().send(req.headers(), res).await;

}
async fn log_init(mut rut: String){
    let path = rut.as_mut_str();
    let file = std::path::Path::new(path);
    let path = file.parent().unwrap().to_str().unwrap();
    let log_file = rolling::daily(path.to_owned() + "/logs/debug", "debug.log")
    .with_max_level(Level::DEBUG).with_min_level(Level::WARN);
    let warn_file = rolling::daily(path.to_owned() + "/logs/wran", "wran.log")
    .with_max_level(Level::WARN).with_min_level(Level::WARN);
    let error_file = rolling::daily(path.to_owned() + "/logs/error", "error.log").with_max_level(Level::ERROR);

    let all_files = log_file.and(error_file).and(warn_file);
    let timer = OffsetTime::new(
        UtcOffset::from_hms(8, 0, 0).unwrap()
    , macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"));
    let timer_clone = timer.clone();
    let a = tracing_subscriber::fmt::Layer::new()
        .with_writer(all_files)
        .with_ansi(false)
        .with_timer(timer);
   
    tracing_subscriber::registry()
    .with(a)
    .with(fmt::layer().with_timer(timer_clone).with_filter(LevelFilter::DEBUG))
    .init();

    tracing::info!("日志初始化成功");
}

fn zip_file(source_dir: &std::path::Path, utc: String) -> String {
    // let utc_datetime: chrono::DateTime<Utc> = Utc::now();
    // let utc  = utc_datetime.format("%Y-%m-%d").to_string();
    let mut filename = utc.to_owned() + ".zip";
    let file_dir = source_dir.to_str().unwrap();
    filename.insert_str(0,file_dir);
    let file = std::fs::File::create(filename.clone()).unwrap();
    // let file = fs::File::create("zip_file").unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    // Walk through the directory and add files to the zip archive.
    let root_path = source_dir.canonicalize().unwrap();
    for entry in WalkDir::new(&root_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // 计算相对于项目根目录的相对路径
        let relative_path = path.strip_prefix(&root_path).unwrap_or(path);
        let name = relative_path.to_str().unwrap();

        if path.is_file() {
            zip.start_file(name, options).unwrap();
            let mut f = fs::File::open(path).unwrap();
            io::copy(&mut f, &mut zip).unwrap();
        } else if path.is_dir() {
            // 添加目录条目，确保目录名以斜杠结尾
            zip.add_directory(name.to_owned() + "/", options).unwrap();
        }
    }


    zip.finish().unwrap();
    
    filename
}

async fn stop_server() {
    let service_name = "MSSQL$SQLEXPRESS";
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access).unwrap();

    let service = service_manager.open_service(&service_name, ServiceAccess::PAUSE_CONTINUE).unwrap();

    println!("Pause {}", service_name);
    let paused_state = service.stop().unwrap();
    println!("{:?}", paused_state.current_state);

    // println!("Resume {}", service_name);
    // let resumed_state = service.resume().unwrap();
    // println!("{:?}", resumed_state.current_state);

}

async fn start_server() {
    let service_name = "MSSQL$SQLEXPRESS";
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access).unwrap();

    let service = service_manager.open_service(&service_name, ServiceAccess::PAUSE_CONTINUE).unwrap();

    println!("Pause {}", service_name);
    let paused_state = service.start(&[OsStr::new("Started from Rust!")]).unwrap();
    

}

async fn encrypt_file(path: &str, utc: String) -> String{
    let file_path =  zip_file(std::path::Path::new(path), utc);
    let mut file = File::open(file_path.clone()).await.unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await.unwrap();
    let key_bytes: &[u8; 32] = b"key$&%Q@KKkaqWTGDJI253TWFCS:::wf";
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(&key);
    let nonce_bytes: &[u8; 12] = b"%@R@Rsaldwlf"; // 12 bytes
    let nonce = Nonce::from_slice(nonce_bytes); // 96-bits; unique per message
    // 加密文件内容
    let encrypted_data = cipher.encrypt(nonce, buffer.as_ref()).expect("encryption failure!");
    let mut fi = std::fs::File::create(file_path.to_owned() + ".crabns").unwrap();
    fi.write_all(&encrypted_data).unwrap();
    fi.flush().unwrap();
    tracing::info!("加密成功");
    let _ = std::fs::remove_file(file_path.to_owned());
    file_path.to_owned() + ".crabns"
}

#[cfg(test)]
mod test {
    use dotenv::dotenv;

    use super::*;

    #[tokio::test]
    async fn test_zip() {
        // encrypt_file("D:\\Program Files (x86)\\axum-sqlite").await;
        assert_eq!(1, 1);
    }

    #[test]
    fn path() {
        let path = std::path::Path::new("D:\\Program Files (x86)\\axum-sqlite");
        let p = path.parent().unwrap().to_str().unwrap();
        println!("{p}");
        assert_eq!("D:\\Program Files (x86)/", p);
    }

    #[test]
    fn istest() {
        let mut path = "D:\\Program Files (x86)\\axum-sqlite".to_string();
        let utc_datetime: chrono::DateTime<Utc> = Utc::now();
        let utc  = utc_datetime.format("%Y-%m-%d").to_string();
        path.push_str(&utc);
        let file = ".zip.crabns";
        let _ = path.push_str(&file);
        let d = std::path::Path::new(&path);
        assert!(d.exists())
    }

    fn test_path() {
        dotenv().ok();
        let path = std::env::var("rustserver").unwrap();
        println!("{path}");
        let de = std::fs::File::create(path);

    }

    #[tokio::test]
    async fn test_stop () {
        stop_server().await
    }
}