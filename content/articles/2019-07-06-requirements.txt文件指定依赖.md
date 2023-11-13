# requirements.txt 文件声明依赖

参考: [pip install](https://pip.pypa.io/en/stable/reference/pip_install/#requirements-file-format)

---

requirements.txt 文件用于声明 Python 依赖，平常所见的格式非常简单：

```txt
nose
nose-cov
beautifulsoup4
```

如果是通过 `pip freeze` 生成，还会指定版本，如

```txt
mccabe==0.4.0
netaddr==0.7.19
networkx==2.2
pathlib2==2.3.2
pbr==5.2.0
pep8==1.7.1
```

pip 会从 PyPI 、配置文件中声明的 index-url 或通过命令行传入的 --index 等 index 站点安装这些包，但根据 pip 文档描述，pip 工具可从以以下四种方式安装依赖：

1. PyPI 或其他 index 站点
2. VCS(版本控制系统，如 Git svn)项目 url
3. 本地项目文件夹
4. 本地或远程归档文件

下面来分别说明四种使用方式

## PyPI

这种方式最为简单，不做赘述，仅给出常用例子

```
# 指定一个版本
project == 1.3

# 指定版本区间
project >=1.2,<2.0

# 使用该版本的兼容发行版
project~=1.4.2

# 6.0 以后的特性，可以指定环境

# Python 版本小于 2.7 时安装 5.4 版本
project == 5.4; python_version < '2.7'

# 仅在 Windows 环境下安装
project; sys_platform == 'win32'
```

完整描述请参考 [PEP-440](https://www.python.org/dev/peps/pep-0440/#version-specifiers)

## VCS

pip 支持 Git, Mercurial, Subversion and Bazaar，因为 Git 最常用，所以这里只描述 pip + Git.

首先需要在运行 pip 命令的机器上安装 git。其次有两种安装方式 [editable](https://pip.pypa.io/en/stable/reference/pip_install/#editable-installs) 和 non-editable

如果使用 `--editable` 或 `-e` 以editable 模式安装，从远端拉取的文件位于 `<cwd>/src/project`(全局安装) 或 `<venv_path>/src/project`(虚拟环境安装)，可以通过 `--src` 选项来覆盖默认值

如果以 non-editable 模式安装，文件会被保存在临时文件中并照常安装，但如果环境中已有满足依赖的包，拉取下来的包将不会覆盖原有的包，除非使用 `--upgrade`

如果使用 vcs, pip 需要通过 `egg=<project_name>` 来指定包名称，如

```txt
-e git://git.exmaple.com/project#egg=my_project
```

如果项目的 `setpy.py` 文件不在根目录下，如下面的项目结构

```
pkg_dir/
    setup.py # setup.py for package pkg
    some_module.py
other_dir/
    some_file
some_other_file
```

还需要指定 subdirectory

```txt
-e git://repo_url/#egg=pkg&subdirectory=pkg_dir
```

除了 git 协议，pip 还支持以下传输协议

git git+http git+https git+ssh git+git git+file

```
[-e] git://git.example.com/project#egg=project
[-e] git+http://git.example.com/project#egg=project
[-e] git+https://git.example.com/project#egg=project
[-e] git+ssh://git.example.com/project#egg=project
[-e] git+git://git.example.com/project#egg=project
[-e] git+file:///home/user/projects/project#egg=project
-e git+git@git.example.com:project#egg=project
```
另外也可以指定分支、tag 和 commit hash

```
[-e] git://git.example.com/project.git@master#egg=project
[-e] git://git.example.com/project.git@v1.0#egg=project
[-e] git://git.example.com/project.git@da39a3ee5e6b4b0d3255bfef95601890afd80709#egg=project
```

如果使用 commit hash 建议使用完整 hash 这样可以减少 git api 调用次数

## 本地文件或归档文件

这两者都比较简单，看例子即可，归档文件可以是 tar.gz 或者是 wheel 文件

```
# 本地文件
[-e] <local_project_path>

# 本地 tar.gz
<path_to_tar.gz>
file:///<absolute_path_to_tar.gz>

# 远程 tar.gz
http[s]://<url_to_tar.gz>
```

wheel 文件用法相同