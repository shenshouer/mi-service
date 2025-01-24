# 小米云服务(rust版)

参考：https://github.com/Yonsm/MiService/

## 1. 介绍
已经实现了最主要的鉴权与核心请求功能，其他功能陆续实现中。

## 2. 使用

### 1. 参考`.env.sample`创建`.env`文件

```shell
# .env
# 日志输出级别
LOG_LEVEL=DEBUG
# 小米账号
MI_USER=xxxx
# 小米密码
MI_PASS=xxxx
# cookie存放路径
MI_TOKEN=/Users/sope/.mi.rs.token
```

### 2. 运行

```shell
# miio 显示账号中的设备列表
cargo r --bin cli list

# MiNA 显示账号中的设备列表
cargo r --bin cli list --mina

# 查看帮助
% cargo r -- --help
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.29s
     Running `target/debug/cli --help`
Usage: cli [OPTIONS] --user <USER> --pass <PASS> <COMMAND>

Commands:
  list  List all devices
  help  Print this message or the help of the given subcommand(s)

Options:
  -l, --log-level <LOG_LEVEL>    日志级别 [env: LOG_LEVEL=] [default: info]
  -u, --user <USER>              Username账号 [env: MI_USER=]
  -p, --pass <PASS>              Password密码 [env: MI_PASS=]
  -t, --token-file <TOKEN_FILE>  Token文件路径 [env: MI_TOKEN=] [default: ~/.mi.token]
  -h, --help                     Print help
  -V, --version                  Print version
```
