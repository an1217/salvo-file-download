use aes_gcm::{aead::{generic_array::GenericArray, Aead}, Aes256Gcm, Key, KeyInit, Nonce};
use chrono::{DateTime, FixedOffset, Local, TimeZone, Timelike, Utc};
use futures::StreamExt;
use tokio::{fs::File, io::AsyncReadExt, time::{sleep_until, Duration, Instant}};
use encrypted_id;
use tokio_cron_scheduler::Job;
use std::{io::Write, sync::Arc};
use salvo::prelude::*;
use time::{macros, UtcOffset};
use tracing::level_filters::LevelFilter;
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, Layer};
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;
#[tokio::main]
async fn main(){
    encrypted_id::init("23t4y567kuydw3456ukjhgfd8*&&%￥#");
    dotenv::dotenv().ok();
    let mut rust = std::env::var("rustclient").expect("请设置环境变量rustclient");
    let _ = std::env::var("serverurl").expect("请设置环境变量serverurl");
    log_init(rust).await;
    // 定时是非东八区时间
    let mut sched = tokio_cron_scheduler::JobScheduler::new().await.unwrap();
    sched.add(
        Job::new_async("0 0 1 * * *", |_uuid, _l| {
            Box::pin(async move {
                download_file().await;
            })
        }).unwrap()
    ).await.unwrap();
    
    download_file().await;
    sched.start().await;
    let acceptor = TcpListener::new("0.0.0.0:22141").bind().await;
    Server::new(acceptor).serve(Router::new()).await;
}


async fn download_file() {
    let client = reqwest::Client::new();
    // let now = chrono::Local::now();
    let utc_datetime: DateTime<Utc> = Utc::now();
    
    let offset = FixedOffset::east(8 * 3600); // 8小时偏移量

    // 将UTC时间转换为东八区时间
    let beijing_time: DateTime<FixedOffset> = utc_datetime.with_timezone(&offset);
    println!("{beijing_time:?}");
    let mut utc  = beijing_time.format("%Y-%m-%d").to_string();
    let naive_date = chrono::NaiveDate::parse_from_str(&utc, "%Y-%m-%d")
        .expect("Failed to parse formatted date");
    
    // 将NaiveDate转换为DateTime<Utc>，并假设时间为午夜（00:00:00）
    let datetime_utc: DateTime<Utc> = Utc.from_utc_date(&naive_date).and_hms(0, 0, 0);
    println!("{datetime_utc:?}");
    // 获取Unix时间戳（秒）
    let timestamp = datetime_utc.timestamp();
 
    let de = encrypted_id::encrypt(timestamp as u64, ":>><<::{3rfsaqwwkeyddadadaw1w*%@^%&@($(@)$").unwrap();
    let mut url = std::env::var("serverurl").unwrap();
    let str = "http://";
    url.push_str(":5800");
    url.insert_str(0, str);

    let response = match client.post(url + "/download")
        .body(de)
        .send()
        .await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Request failed: {}", e);
                return;
            }
        };
    // response.bytes().await.unwrap()
    if response.status().is_success() {
        // let all = response.headers().clone();
        // println!("{all:?}");

        let filename = response.headers().clone();
        let filename = filename.get("content-disposition");
        match filename {
            Some(e) => {
                let mut path = std::env::var("rustclient").unwrap();
                println!("{e:?}");
                let str: Vec<&str> = e.to_str().unwrap().split("filename=").collect();
                println!("{str:?}");
                let clonew = response.bytes().await.unwrap();
                let filename = str[1];
                path.push_str("\\");
                path.push_str(filename);

                tokio::fs::write(path.clone(), clonew).await;
                decrypt_file(&path).await;
            },None => {
                tracing::error!("未获取到请联系管理员");
            }   
        };
    
    }
    
    // println!("{response:?}");
    // response
    

}

async fn decrypt_file(path: &str){
    // let path = "./2025-02-21.zip.crabns";
    let mut file = File::open(path).await.unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await.unwrap();
    let key_bytes: &[u8; 32] = b"key$&%Q@KKkaqWTGDJI253TWFCS:::wf";
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(&key);
    let nonce_bytes: &[u8; 12] = b"%@R@Rsaldwlf"; // 12 bytes
    let nonce = Nonce::from_slice(nonce_bytes); // 96-bits; unique per message
    // 解密文件内容
    let encrypted_data = cipher.decrypt(nonce, buffer.as_ref()).expect("encryption failure!");
    let splite:Vec<&str> = path.split(".crab").collect();
    let mut fi = std::fs::File::create(splite[0]).unwrap();
    fi.write_all(&encrypted_data).unwrap();
    fi.flush().unwrap();
    let _ = std::fs::remove_file(path);
    tracing::info!("解密成功");
    
}
async fn log_init(mut rut: String){
    let path = rut.as_mut_str();
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

#[cfg(test)]
mod tests {
    use super::*;

    // #[tokio::test]
    // async fn test_add() {
    //     decrypt_file("path").await;
    // }

    #[test]
    fn test() {
    let mut url = "192.168.2.61".to_string();
    let str = "http://";
    url.push_str(":5800");
    url.insert_str(0, str);
    assert_eq!("http://192.168.2.61:5800", &url);
    }
}