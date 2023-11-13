# PowerShell 配置

---

## code 命令打开远程目录

因为我在家有两台机器，一台玩游戏的 Windows 机器， 一台用于开发的 NUC + 深度。通过 vscode remote-ssh 连接到
NUC 进行开发，由于工作目录很多，右键 vscode 图表显示的快速打开无法覆盖所有工作目录，因此经常需要先打开 remote-ssh
扩展，然后选择 host 和上面的工作目录。如果从来没有用 vscode 打开过远程目录，那就更麻烦了。我希望能像本地 vscode
一样通过 `code <dir>` 直接打开 vscode(在 wsl 中也可以通过 code 直接打开 windows 上的 vscode)。

不过好在 `code` 命令支持命令行打开远程目录。[Github - Issue](https://github.com/microsoft/vscode-remote-release/issues/656#issuecomment-584209378)

```bash
code --remote ssh-remote+<remote_host> <remote_path>
```

我们可以在 `$PROFILE` 中定义 `CodeRemote` 函数帮助我们简化打开远程文件流程

```ps
function CodeRemote {
    <#
        .Description
        code 命令打开远程文件
    #>
    param (
        # 远端服务器 [user@]hostname
        [Parameter(Mandatory = $true)]
        [string]
        $remote,


        # 远端目录路径
        [Parameter(Mandatory = $true)]
        [string]
        $path
    )
    $path = ssh $remote echo $path
    code --remote ssh-remote+$remote $path
}
```

通过参数指令 `Parameter(Mandatory = $true)]` 我们把两个参数设置为必选参数，同时为他们指定参数类型。

在通过 `code` 命令打开以前，我们首先要展开传入的 `$path`, 因为在 pwsh 下，`~` 不会被展开为 `$HOME`, 而 `$HOME` 则会被展开为 pwsh 下的 home 目录，与我们要求不符。所以需要通过运行一个 shell 命令展开文件路径。

## 在 pwsh 启动浏览器

对于 Chrome 和 MS Edge(新版) 可以通过文件快捷方式找到对应的 exe 文件，接着可以通过 `Start-Process` 命令运行 exe 文件打开浏览器，此外启动时支持传入一个 url 参数，指定打开的新标签页网址。

```ps
function open-link {
    <#
        .Description
        使用 MS Edge 打开连接
    #>
    param (
        # 网页地址
        [string]
        $url
    )
    $edge = "C:\Users\rookie\AppData\Local\Microsoft\Edge SXS\Application\msedge.exe"
    Start-Process $edge $url
}
```

## 在 pwsh 通过浏览器搜索

在之前 `open-link` 基础上可以我们可以先拼接出某个搜索引擎的 url, 接着通过浏览器打开那个 url, 即可实现搜索功能。因为不同搜索引擎搜索结果差异较大，所以需要提供多个搜索引擎，由 `--engine` 选项传入。

因为 `--engine` 参数值有限，我们通过将它的类型设置为某个枚举类型即可(PowerShell 支持枚举类型)

```ps
enum SearchEngine {
    Doge
    Bing
    Google
    Baidu
}

function search-thing {
    <#
        .Description
        使用浏览器搜索
    #>
    param (
        # 搜索关键字
        [parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [string]
        $keyword,

        # 指定搜索引擎
        [SearchEngine]
        $engine = [SearchEngine]::Doge
    )
    switch ($engine) {
        { [SearchEngine]::Doge } { $url = "https://www.dogedoge.com/results?q=$keyword"; break }
        { [SearchEngine]::Bing } { $url = "https://cn.bing.com/search?q=$keyword"; break }
        { [SearchEngine]::Google } { $url = "https://google.com/search?q=$keyword"; break }
        { [SearchEngine]::Baidu } { $url = "https://www.baidu.com/s?wd=$keyword"; break }
        Default { $url = "https://www.dogedoge.com/results?q=$keyword" }
    }
    open-link $url
}
```

可以看到整个函数并不复杂，先定义一个为参数准备的枚举类型，接着匹配 `$engine` 拼接 url，最后通过 `open-link` 打开 url。

还有一个值得注意的点是， `$keyword` 参数添加了额外的指令 `ValueFromPipeline = $true`, 这意味着可以通过

```ps
echo "github" | search-thing
```

这样的管道传递参数


## 传入任意参数

除了在函数定义是指定的参数，PowerShell 还支持通过 `$args` 这个特殊变量，传递所有参数，而不进行校验，提供了很强的灵活性。

在 Win10 中，通过应用商店下载的 Python（这是win10的默认安装方式）运行 `pip install` 安装的包不会放入环境变量中，这导致通过 `pip install pipenv` 后无法在命令行执行 `pipenv`，必须通过 `python -m pipenv` 唤起。

当然，这个问题可以通过把脚本路径添加到环境变量解决，但那样有些麻烦。

如果你尝试用 `Set-Alias` 将 pipenv 设置为 `python -m pipenv` 你会发现无法传入 pipenv 参数。

这时候 `$args` 就可以排上用场

```ps
function pipenv {
    python.exe -m pipenv $args
}
```

`pipenv` 现在实际上是一个 powershell 函数，它执行 `python.exe -m pipenv ` 然后把所有参数一股脑传给 pipenv 模块，这就相当于我们直接调用 `pipenv`。
