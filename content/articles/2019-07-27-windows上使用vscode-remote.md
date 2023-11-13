# Winddows 上使用VSCode Remote 插件进行远程开发

---

直到 VSCode remote-ssh 插件出来和我买了一台 NUC 机器之前，我一直在维护两套开发环境，
一套防在公司的 CentOS 虚拟机上，另一套则是家里的 Windows 机器。因为代码同步以及 Windows
和 Linux 系统差异等问题，维护两套环境非常麻烦。但是现在通过 [VSCode remote-ssh](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.vscode-remote-extensionpack) +
VPN 的方式就可以在 Windows 上使用 NUC 机器进行开发。

## 配置远程机器

这个方案也适用于虚拟机作为远程机器，只要它能够通过 SSH 远程访问即可。这里我使用 mosh 作为
openssh 替代方案，因为我在 windows 上使用的 [FluentTerminal](https://github.com/felixse/FluentTerminal)
对 mosh 有更好的支持。这是我在 Windows 上最满意的终端，颜值不错，速度比 Hyper 要快，而且支持
iterm 颜色方案导入

![fluent-terminal.png](/static/assets/2020/fluent-terminal.png)

远程机器上可以通过下面的方式安装 mosh-server

```bash
# CentOS
# 需要安装 epel-release
sudo yum -y install epel-release
sudo yum update
sudo yum -y install mosh

# Ubuntu 或 deepin
sudo apt -y install mosh

# 启动 mosh-server
mosh-server
```

在远程机器上我习惯用 oh-my-zsh，可是远程连接上去的时候 `PATH` 环境变量与直接登录时有差异
导致远程连接时 `cargo` `procs` 等命令无法使用。简单粗暴的解决办法时将直接登录时的 `PATH`
写入 `~/.zshrc` 文件

```bash
# ~/.zshrc
PATH=$PATH:/home/rookie/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/home/rookie/bin:/bin
```

## 配置本地机器

建议在将本地机器上的 `id_rsa.pub` 文件内容复制到 `~/.ssh/authorized_keys` 中，这样可以
避免每次连接时都需要输入密码。

接着配置 ssh config 文件示例配置如下

```conf
# ~/.ssh/config
Host NUC
    HostName 172.16.130.38
    User rookie
    IdentityFile ~/.ssh/id_rsa
```

接着在本地的 VSCode 下载 vscode-remote 插件，虽然还处于 preview 阶段，但正式版 VSCode
现在也可以直接安装了。

![remote.png](/static/assets/2020/remote.png)

安装完成后应该会在左下角有个连接图标

![image.png](/static/assets/2020/image.png)

点击然后选择 remote-SSH: Connection to remote host，然后在 Host 列表选择之前配置好的
NUC，等待连接完成。

连接完成后 VSCode 会打开一个新窗口，窗口左下角的远程连接图标显示当前窗口已连接到 NUC

![NUC.png](/static/assets/2020/NUC.png)

选择打开文件打开远程机器上的一个文件夹即可开始在远端机器编辑代码

![image.png](/static/assets/2020/image.png)

值得一提的是在这个窗口打开的命令行也是在远程机器上的，不需要打开后再手动登录  :+1: 。

不过需要注意的是远程的 VSCode 不会同步本地插件，需要再次安装，安装后的插件右下角也会带远程连接
的图标

![image.png](/static/assets/2020/image.png)
