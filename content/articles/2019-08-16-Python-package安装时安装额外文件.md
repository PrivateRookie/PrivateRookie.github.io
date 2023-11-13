# Python 分发包中添加额外文件

---

在制作一个 Python 分发包时经常需要把一些文件添加到包中。最常见的例子是你希望通过 `pip install`
命令安装 Python 包时会在 `/etc/` 等目录下自动添加默认配置文件，由此可以让 Python
安装完成就可以工作，同时也可以给用户提供配置样例参考。

参考 [Installing Additional Files](https://docs.python.org/3/distutils/setupscript.html#installing-additional-files)

如果使用 setuptools，在 `setup.py` 文件中可以通过 `data_files` 配置项配置分发包的额外文件，
格式为：`(<安装位置>, [<文件1>, <文件2>, ...])`

```python
# setup.py
from setuptools import setup

setup(...,
      data_files=[('bitmaps', ['bm/b1.gif', 'bm/b2.gif']),
                  ('config', ['cfg/data.cfg'])],
    )
```

上面的示例`<安装位置>`使用了相对路径，在安装时会根据安装前缀如 `sys.prefix`(系统级安装) 和
`site.USER_BASE`(用户级安装)解释为绝对路径。虽然也可以使用绝对路径，但不推荐这么做，因为
这与 wheel 格式的分发包不兼容。文件路径同样也是相对路径，其路径起点为 `setup.py` 文件所在目录，
即项目的根目录，注意文件不能被重命名。

如果使用 [pbr](https://docs.openstack.org/pbr/latest/) 帮助打包，声明额外文件的方法与上述类似，
需要在 `setup.cfg` 如下配合

```cfg
[files]
packages =
    pbr
data_files =
    etc/pbr = etc/*
    etc/init =
        pbr.packaging.conf
        pbr.version.conf
```