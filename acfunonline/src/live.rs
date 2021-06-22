use acfunliveapi::{
    client::ApiClientBuilder,
    response::{LiveList, MedalList},
};
use acfunlivedanmaku::client::DanmakuClient;
use anyhow::{anyhow, bail, Result};
use std::{collections::HashMap, time::Duration};
use tokio::{sync::mpsc, time::sleep};

/// 主播数据
#[derive(Clone, Debug)]
struct Liver {
    uid: i64,
    nickname: String,
}

/// 主播们
type Livers = HashMap<i64, Liver>;

/// 命令
#[derive(Clone, Debug)]
enum Command {
    LiverRooms(Livers),
    Medals(Livers),
    Delete(i64),
}

/// 直播挂牌子
pub async fn keep_online(account: String, password: String) -> Result<()> {
    let api_client = ApiClientBuilder::default_client()?
        .user(account, password)
        .build()
        .await?;
    let (tx, mut rx) = mpsc::channel::<Command>(10);

    let live_room = async {
        let mut num = 0;
        loop {
            match api_client.get::<LiveList>().await {
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
                    let _ = tx.try_send(Command::LiverRooms(livers));
                }
                Err(e) => {
                    num += 1;
                    if num >= 5 {
                        bail!("live list error: {}", e);
                    }
                }
            }
            sleep(Duration::from_secs(30)).await;
        }

        #[allow(unreachable_code)]
        Ok(())
    };

    let medal = async {
        sleep(Duration::from_secs(5)).await;
        let mut num = 0;
        loop {
            match api_client.get::<MedalList>().await {
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
                    let _ = tx.try_send(Command::Medals(livers));
                }
                Err(e) => {
                    num += 1;
                    if num >= 5 {
                        bail!("medal list error: {}", e);
                    }
                }
            }
            sleep(Duration::from_secs(30)).await;
        }

        #[allow(unreachable_code)]
        Ok(())
    };

    let danmaku = async {
        let mut living: Livers = HashMap::new();
        let mut online: Livers = HashMap::new();
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::LiverRooms(livers) => living = livers,
                Command::Medals(livers) => {
                    for (_, liver) in livers {
                        if living.get(&liver.uid).is_some() && online.get(&liver.uid).is_none() {
                            if let Ok(client) =
                                DanmakuClient::from_api_client(&api_client, liver.uid).await
                            {
                                let tx = tx.clone();

                                #[cfg(not(all(
                                    not(debug_assertions),
                                    target_os = "windows",
                                    feature = "gui"
                                )))]
                                println!("online: {}", liver.nickname);

                                let _ = online.insert(liver.uid, liver.clone());
                                let _ = tokio::spawn(async move {
                                    let _ = client.danmaku().await;
                                    let _ = tx.send(Command::Delete(liver.uid)).await;

                                    #[cfg(not(all(
                                        not(debug_assertions),
                                        target_os = "windows",
                                        feature = "gui"
                                    )))]
                                    println!("stop: {}", liver.nickname);
                                });
                            }
                        }
                    }
                }
                Command::Delete(uid) => {
                    let _ = online.remove(&uid);
                }
            }
        }

        Result::<()>::Err(anyhow!("stop keeping online"))
    };

    tokio::try_join!(live_room, medal, danmaku).map(|_| ())
}
