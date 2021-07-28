use acfunliveapi::{
    client::{ApiClient, ApiClientBuilder},
    http::HttpClient,
    response::{LiveList, MedalList},
};
use acfunlivedanmaku::{
    client::DanmakuClient, futures::stream::StreamExt, websocket::WebSocketClient,
};
use anyhow::{anyhow, bail, Result};
use std::{collections::HashMap, time::Duration};
use tokio::{sync::mpsc, time::sleep, time::timeout};

const ERROR_NUM: usize = 10;
const LOOP_INTERVAL: Duration = Duration::from_secs(30);
const TIMEOUT: Duration = Duration::from_secs(10);

/// 主播数据
#[derive(Clone, Debug)]
struct Liver {
    uid: i64,
    nickname: String,
}

/// 主播们的数据
type Livers = HashMap<i64, Liver>;

/// 命令
#[derive(Clone, Debug)]
enum Command {
    LiverRooms(Livers),
    Medals(Livers),
    Delete(i64),
}

/// 获取直播间列表
async fn live_room(client: &ApiClient<HttpClient>, tx: &mpsc::Sender<Command>) -> Result<()> {
    let mut num = 0;
    loop {
        match client.get::<LiveList>().await {
            Ok(list) => {
                num = 0;
                let mut livers: Livers = HashMap::with_capacity(list.live_list.len());
                for live in list.live_list {
                    let _ = livers.insert(
                        live.author_id,
                        Liver {
                            uid: live.author_id,
                            nickname: live.user.name,
                        },
                    );
                }
                if tx.try_send(Command::LiverRooms(livers)).is_err() {
                    log::error!("failed to send LiverRooms");
                }
            }
            Err(e) => {
                num += 1;
                if num >= ERROR_NUM {
                    bail!("live list error: {}", e);
                }
            }
        }
        sleep(LOOP_INTERVAL).await;
    }
}

/// 获取用户拥有的守护徽章
async fn medal(client: &ApiClient<HttpClient>, tx: &mpsc::Sender<Command>) -> Result<()> {
    sleep(Duration::from_secs(5)).await;
    let mut num = 0;
    loop {
        match client.get::<MedalList>().await {
            Ok(list) => {
                num = 0;
                let mut livers: Livers = HashMap::with_capacity(list.medal_list.len());
                for medal in list.medal_list {
                    let _ = livers.insert(
                        medal.uper_id,
                        Liver {
                            uid: medal.uper_id,
                            nickname: medal.uper_name,
                        },
                    );
                }
                if tx.try_send(Command::Medals(livers)).is_err() {
                    log::error!("failed to send Medals");
                }
            }
            Err(e) => {
                num += 1;
                if num >= ERROR_NUM {
                    bail!("medal list error: {}", e);
                }
            }
        }
        sleep(LOOP_INTERVAL).await;
    }
}

/// 获取有守护徽章的直播间的弹幕
async fn all_danmaku(
    client: &ApiClient<HttpClient>,
    tx: &mpsc::Sender<Command>,
    rx: &mut mpsc::Receiver<Command>,
) -> Result<()> {
    let mut living: Livers = HashMap::new();
    let mut online: Livers = HashMap::new();
    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::LiverRooms(livers) => living = livers,
            Command::Medals(livers) => {
                for (_, liver) in livers {
                    if living.get(&liver.uid).is_some() && online.get(&liver.uid).is_none() {
                        if let Ok(client) = DanmakuClient::from_api_client(&client, liver.uid).await
                        {
                            let tx = tx.clone();
                            log::info!("[{}] online: {}", liver.uid, liver.nickname);
                            let _ = online.insert(liver.uid, liver.clone());
                            let _ = tokio::spawn(danmaku(client, tx, liver));
                        }
                    }
                }
            }
            Command::Delete(uid) => {
                let _ = online.remove(&uid);
            }
        }
    }

    Err(anyhow!("stop keeping online"))
}

/// 获取弹幕
async fn danmaku(
    mut client: DanmakuClient<WebSocketClient>,
    tx: mpsc::Sender<Command>,
    liver: Liver,
) {
    loop {
        match timeout(TIMEOUT, client.next()).await {
            Ok(option) => match option {
                Some(Err(e)) => {
                    log::error!("[{}] getting danmaku error: {}", liver.uid, e);
                    break;
                }
                None => break,
                _ => {}
            },
            Err(_) => {
                log::error!("[{}] getting danmaku timeout", liver.uid);
                break;
            }
        }
    }
    if let Err(e) = client.close().await {
        log::error!(
            "[{}] failed to close client WebSocket connection: {}",
            liver.uid,
            e
        );
    }
    if tx.send(Command::Delete(liver.uid)).await.is_err() {
        log::info!("[{}] failed to send Delete", liver.uid);
    }

    log::info!("[{}] stop: {}", liver.uid, liver.nickname);
}

/// 直播挂牌子
pub async fn keep_online(account: String, password: String) -> Result<()> {
    let client = ApiClientBuilder::default_client()?
        .user(account, password)
        .build()
        .await?;
    let (tx, mut rx) = mpsc::channel::<Command>(10);

    tokio::try_join!(
        live_room(&client, &tx),
        medal(&client, &tx),
        all_danmaku(&client, &tx, &mut rx)
    )
    .map(|_| ())
}
