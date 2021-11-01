# Fedora CoreOS 尝鲜

---

# 使用 Fedora CoreOS

使用 virt-manager 作为虚拟机管理器

## 准备资源

需要2个镜像, installer iso 作为安装引导，raw.gz 作为真正的系统镜像。

进入 [Download Fedora CoreOS](https://getfedora.org/en/coreos/download/),

下载 install iso, 保存为 fedora-coreos_installer.iso, 下载 raw 格式镜像, 解压并重新压缩为 gzip
格式，保存为 fedora-cores.raw.gz

coreos 使用 igniton 而不是 cloud-init 作为机器初始化配置工具, coreos 提供了专门的工具，将更加易读 yaml 格式的
配置项转换 fedora-coreos 标准 ignition 文件。

进入 [download fcct](https://github.com/coreos/fcct/releases) 下载对应平台的软件, 以 linux 为例。

```bash
mv fcct-x86_64-unknown-linux-gnu /usr/bin/local/fcct
chmod +x /usr/bin/local/fcct
```

ignition 支持配置用户、用户组, 磁盘设置, systemd 服务配置等, 在 fcct 中可用的所有配置项可以参考:
[Configuration Specification v1.0.0](https://github.com/coreos/fcct/blob/master/docs/configuration-v1_0.md)

这里给出配置 root 用户密码和远程登录的例子

```yaml
variant: fcos
version: 1.0.0
passwd:
  users:
    - name: root
      password_hash: $6$rounds=4096$XGD8LIedQn1ew$UsWJLqM59OSVCJDFGlyMsUpifAG./BAbY03mdIciLSCc7namSJI9Tx/ak1UlHHgupH8u8neqq2IxzKS37FVO4/
      ssh_authorized_keys:
        - ssh-rsa AAAAB3NzaC1yc2EAAAADAQA...
```

password_hash 可以通过 `mkpasswd --method=SHA-512 --rounds=4096` 生成,
ssh_authorized_keys 可以通过 `cat ~/.ssh/id_rsa.pub` 查看

接着使用 fcct 将其转换为 json 格式的标准 ignition 文件 `ignition.json`

```json
{
    "ignition": {
        "config": {
            "replace": {
                "source": null,
                "verification": {}
            }
        },
        "security": {
            "tls": {}
        },
        "timeouts": {},
        "version": "3.0.0"
    },
    "passwd": {
        "users": [
            {
                "name": "root",
                "passwordHash": "$6$rounds=4096$XGD8LIedQn1ew$UsWJLqM59OSVCJDFGlyMsUpifAG./BAbY03mdIciLSCc7namSJI9Tx/ak1UlHHgupH8u8neqq2IxzKS37FVO4/",
                "sshAuthorizedKeys": [
                    "ssh-rsa AAAAB3NzaC1yc2EAAAADAQA..."
                ]
            }
        ]
    },
    "storage": {},
    "systemd": {}
}
```

把 fedora-coreos.raw.gz 和 ignition.json 文件放到同一个文件夹内, 启动
一个简单的 http 服务用于安装

```bash
python3 -m http.server 8100
```

## 安装使用

打开 virt-manager, 创建虚拟机, 选择使用 ISO 映像或光驱安装, 选择 installer.iso 镜像,
内存, CPU 和存储大小根据宿主机配置自行改变, 网络选择默认NAT模式, 点击完成.

等待引导程序启动, 会看到如下界面

[![install-coreos.png](https://i.postimg.cc/zXWW539x/install-coreos.png)](https://postimg.cc/crx6B113)

选择 Install Fedora CoreOS, 接着在如下页面输入镜像 URL `http://<IP>:8100/fedora-coreos.raw.gz`

[![intpu-image-url.png](https://i.postimg.cc/t4j6gpbp/intpu-image-url.png)](https://postimg.cc/JGp06wt2)

按 Enter 继续, 输入 ignition 文件 URL `http://<IP>:8100/ignition.json`

[![intpu-image-url.png](https://i.postimg.cc/t4j6gpbp/intpu-image-url.png)](https://postimg.cc/JGp06wt2)

按 Enter 完成安装. 镜像安装成功将会自动重启, 这时候就可以在宿主机上通过
ssh 直接登录

[![coreos-login.png](https://i.postimg.cc/wBWRFNmj/coreos-login.png)](https://postimg.cc/QBTMdHm2)

至此 Fedora CoreOS 安装完毕, 在 Fedora CoreOS 中安装软件都是使用容器安装, 推荐使用 podman 进行管理, 详细使用参考 [Reintroduction of Podman](https://www.projectatomic.io/blog/2018/02/reintroduction-podman/)