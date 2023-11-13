# OpenStack允许 root 用户使用 ssh key 登录

参考: [Enable Root Login Over SSH With Cloud-Init on OpenStack](https://mcwhirter.com.au/craige/blog/2015/Enable_Root_Login_Over_SSH_With_Cloud-Init_On_OpenStack/)

---

默认情况下如果在创建虚拟机时将 ssh 密钥注入后尝试使用 root 用户登录会提示：

```bash
Please login as the user "centos" rather than the user "root".
```

此时需要移除 `/root/.ssh/authorized_keys` 第一行的如下内容：

```bash
no-port-forwarding,no-agent-forwarding,no-X11-forwarding,command="echo 'Please login as the user \"centos\" rather than the user \"root\".';echo;sleep 10"
```

并检查 `/etc/ssh/sshd_config` 中的 `PermitRootLogin` 配置项，确保其设置如下：

```conf
PermitRootLogin without-password
```

随后重启 sshd 服务。

---

如果不想在每次创建成功后手动修改，需要修改镜像中的 cloud-init 配置项，
修改 `/etc/cloud/cloud.cfg` 如下配置项：

```cfg
disable_root: false
# 或者
disable_root: 0
```