# miniserve 简单美观的文件服务器

---

如果想建立一个简单静态文件或目录服务器，通常可以用 Python 实现，而且非常简单

```bash
# Python 2
python -m SimpleHTTPServer <port>

# Python 3
python3 -m http.server <port>
```

一般情况下，这就够用了，但如果这样的服务器在浏览器提供的界面有些简陋，而且不提供认证服务。更复杂
的实现方法是使用 Nginx，但 Nginx 的配置相对繁琐，这里推荐一个使用 Rust 基于 [Actix](https://actix.rs/)
框架实现静态文件或文件夹服务器 [miniserve](https://github.com/svenstaro/miniserve)，demo如下

![miniserve.png](https://i.postimg.cc/MGpkBQ5p/miniserve.png)

除了更加漂亮的界面和基本用户认证外 miniserve 还支持如下功能

- 将当前文件夹压缩后下载
- 界面上传文件（可配置）
- 支持监听多网卡
- 自动更改 MIME
- 超级快（powered by Rust and Actix）

---

## 下载

在[发行版界面](https://github.com/svenstaro/miniserve/releases)找到操作系统对应的版本，文件很小，最大的 osx 也仅有 3.2MB。

### Linux
```bash
sudo curl -L https://github.com/svenstaro/miniserve/releases/download/v0.4.1/miniserve-linux-x86_64 -o /usr/local/bin/miniserve
sudo chmod +x /usr/local/bin/miniserve
```

### OSX
```bash
sudo curl -L https://github.com/svenstaro/miniserve/releases/download/v0.4.1/miniserve-osx-x86_64 -o /usr/local/bin/miniserve
sudo chmod +x /usr/local/bin/miniserve
```

### Windows
windows 下载好 exe 文件可直接运行

### Cargo
如果电脑上安装了 Rust 和 Cargo，也可以通过 Cargo 安装，但由于 miniserve
仅支持 nightly channel，所以你得先切换到 nightly channel

```bash
rustup toolchain add nightly
rustup default nightly

cargo install miniserve
```

### Docker
miniserve 在 docker hub 上的镜像名为 [svenstaro/miniserve](https://hub.docker.com/r/svenstaro/miniserve)

```bash
docker pull svenstaro/miniserve
```

## 使用
全部参数如下
```bash
miniserve --help
miniserve 0.4.1
Sven-Hendrik Haase <svenstaro@gmail.com>, Boastful Squirrel <boastful.squirrel@gmail.com>
For when you really just want to serve some files over HTTP right now!

USAGE:
    miniserve [FLAGS] [OPTIONS] [--] [PATH]

FLAGS:
    -u, --upload-files       Enable file uploading
    -h, --help               Prints help information
    -P, --no-symlinks        Do not follow symbolic links
    -o, --overwrite-files    Enable overriding existing files during file upload
        --random-route       Generate a random 6-hexdigit route
    -V, --version            Prints version information
    -v, --verbose            Be verbose, includes emitting access logs

OPTIONS:
    -a, --auth <auth>                    Set authentication (username:password)
    -c, --color-scheme <color_scheme>    Default color scheme [default: Squirrel]  [possible values:
                                         Archlinux, Zenburn, Monokai, Squirrel]
    -i, --if <interfaces>...             Interface to listen on
    -p, --port <port>                    Port to use [default: 8080]

ARGS:
    <PATH>    Which path to serve
```

### 服务某个文件夹
```bash
miniserve some_dir
```

### 服务单个文件
```bash
miniserve file
```

### 启用用户认证
`--auth user:passwd` 可以提供简单用户认证服务
```bash
miniserve --auth joe:123 some_dir
```

### 在根目录后添加随机6位URL
```bash
miniserve -i 192.168.0.1 --random-route some_dir
# 服务器URL为 http://192.168.0.1/c78b6
```

### 绑定多张网卡
```bash
miniserve -i 192.168.0.1 -i 10.13.37.10 -i ::1 some_dir
```

### 使用容器
```bash
# 后台运行
docker run -d --name miniserve -p 8080:8080 --rm svenstaro/miniserve some_dir

# 前台运行
docker run --it --name miniserve -p 8080:8080 --rm svenstaro/miniserve some_dir
```
