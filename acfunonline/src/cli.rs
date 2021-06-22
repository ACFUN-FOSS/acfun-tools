use crate::live::keep_online;
use anyhow::Result;
use clap::{App, Arg};

/// CLI启动
pub async fn cli() -> Result<()> {
    let matches = App::new("AcFun Live Online")
        .version("0.1.0")
        .author("orzogc")
        .about("Keeping online in AcFun lives which are in your medal list")
        .arg(
            Arg::with_name("account")
                .short("a")
                .long("account")
                .value_name("ACCOUNT")
                .help("AcFun account's phone number or email")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("password")
                .short("p")
                .long("password")
                .value_name("PASSWORD")
                .help("AcFun account's password")
                .takes_value(true),
        )
        .get_matches();
    let account = match matches.value_of("account") {
        Some(s) => s.to_string(),
        None => casual::prompt("AcFun account's phone number or email: ").get(),
    };
    let password = match matches.value_of("password") {
        Some(s) => s.to_string(),
        None => rpassword::read_password_from_tty(Some("AcFun account's password: "))?,
    };

    keep_online(account, password).await
}
