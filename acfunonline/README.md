# acfunonline
AcFun直播间挂牌子工具

本工具不会保存用户的帐号和密码

## 编译
### 命令行版本
```
cargo build --release
```

编译好的可执行文件在target/release

### GUI版本
```
cargo build --release --no-default-features --features gui
```

编译好的可执行文件在target/release

## 使用方法
### 命令行版本
```
acfunonline -a AcFun帐号手机号或邮箱 -p AcFun帐号密码
```

或者直接运行acfunonline，按照提示输入AcFun帐号手机号或邮箱和密码登陆

### GUI版本
直接运行，按照提示输入AcFun帐号手机号或邮箱和密码登陆
