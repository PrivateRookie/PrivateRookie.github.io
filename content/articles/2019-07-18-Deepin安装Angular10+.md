# Deepin安装Angular10+

---

## 安装 nodejs

因为深度系统默认源中 nodejs 版本过低，改用 nodejs 官方源

```bash
curl -sSL https://deb.nodesource.com/gpgkey/nodesource.gpg.key | \
sudo apt-key add -
echo "deb https://mirrors.tuna.tsinghua.edu.cn/nodesource/deb_10.x stretch main"\
 | sudo tee /etc/apt/sources.list.d/nodesource.list
echo "deb-src https://mirrors.tuna.tsinghua.edu.cn/nodesource/deb_10.x stretch main"\
 | sudo tee -a /etc/apt/sources.list.d/nodesource.list
sudo apt-get update
sudo apt-get install nodejs
```

## 安装并配置 npm

```bash
sudo apt install npm
# 改用淘宝镜像
npm install -g cnpm --registry=https://registry.npm.taobao.org
```

更多 npm 镜像配置信息参考 [npm-taobao](https://npm.taobao.org/)

## 安装 Angular 10 cli

```bash
cnpm i -g install @angular/cli
```