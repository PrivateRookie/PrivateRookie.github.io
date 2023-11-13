# 使用OSProfiler对OpenStack进行性能测量

---

## 配置服务

如果要在 OpenStack 项目中使用 OSprofiler 进行性能跟踪，大部分项目已经帮我们
初始化了 OSprofiler, 只需要在配置文件中添加 `[profiler]` 配置项即可。

OSprofiler 支持使用多种 collector 存储数据，默认使用消息队列+Ceilometer(由oslo.messaging提供驱动)。
我这里使用 mongodb 作为 collection，原因只是我不喜欢 Ceilometer 和 Mongodb 更方便查询的GUI。
使用 mongodb 前需先安装 python mongodb 驱动 `pip install pymongo`。

创建 mongodb 数据库，假设数据库 Host IP 为 172.16.140.116。

```bash
mkdir osprofiler_db && cd osprofiler_db

docker run \
-p 27017:27017 \
-v $pwd/data/db \
--name osprofiler_db \
-d mongo:latest
```

```conf
[profiler]
enabled = True
connection_string = mongodb://172.16.40.116:27017
# hmac_keys 将会被用作 `--osprofiler` 的参数，可以任意指定一串字符
hmac_keys = 123
# 控制跟踪的内容
trace_wsgi_transport = True
trace_message_store = True
trace_management_store = True
```

如果使用消息队列+Ceilometer，还需要配置Ceilometer，使其不会丢弃OSprofiler发送的消息。

```conf
[oslo_messaging_notifications]
topics = notifications, profiler
```

关于应该在哪些点进行跟踪，OSprofiler 的文档中建议：

1. 所有的 HTTP 请求 - 发出了什么 HTTP 请求，请求时长（服务延迟），请求涉及项目
2. 所有 RPC 请求 - 有助于理解某项目中不同服务请求时长，这对于发现项目性能瓶颈非常有用
3. 所有 DB API 请求 - 某些情况下慢 DB 查询是性能瓶颈。DB 查询耗时是非常有用的数据。
4. 所有驱动调用 - 在有 Nova, Cinder 或其他三方驱动情况下，跟踪驱动性能
5. 所有 SQL 请求（默认关闭，因为会产生很多消息）

## 使用

假设想跟踪某个 API 请求，只需要在 `openstack` 命令添加 `--osprofile hmac_key`，假设 Glance 已经配置好了

```bash
openstack image list --osprofile 123
```

命令运行完成后会在终端打印如下内容

```bash
Trace ID: 2902c7a3-ee18-4b08-aae7-4e34388f9352
Display trace with command:
osprofiler trace show --html 2902c7a3-ee18-4b08-aae7-4e34388f9352
```

接着使用 `osprofiler` 命令将报告以 html 格式打印出来。

```bash
# 记得指定后端
osprofiler trace show --html 2902c7a3-ee18-4b08-aae7-4e34388f9352 \
--connection-string "mongodb://172.16.140.116:27017" \
-o "report.html"

# 也可以输出 json 格式报告
osprofiler trace show --json 2902c7a3-ee18-4b08-aae7-4e34388f9352 \
--connection-string "mongodb://172.16.140.116:27017" \
-o "report.json"
```

使用浏览器打开 report.html 即可看到报告

## 添加自定义跟踪点

如果没有初始化 osprofiler，即使代码上打上跟踪点，osprofiler 也不会发送任何消息。如果在使用的项目中没有进行初始化，可以参考下面的方法进行初始化。

```python
if CONF.profiler.enabled:
    osprofiler_initializer.init_from_conf(
        conf=CONF,
        contex={},
        project="cinder",
        service=binary,
        host=host
    )
```

有 5 种方法在代码中打跟踪点

```python
from osprofiler import profiler

# 手动设置开启和关闭
def some_func():
    profiler.start("point_name", {"any_key": "with_any_value"})
    # your code
    profiler.stop({"any_info_about_point": "in_this_dict"})


# 使用装饰器
@profiler.trace("point_name",
                info={"any_info_about_point": "in_this_dict"},
                hide_args=False)
def some_func2(*args, **kwargs):
    # 如果需要隐藏 profile 信息中的 args, 使用 hide_args=True
    pass

# 使用上下文管理器
def some_func3():
    with profiler.Trace("point_name",
                        info={"any_key": "with_any_value"}):
        # some code here

# 跟踪某个类方法
@profiler.trace_cls("point_name", info={}, hide_args=False,
                    trace_private=False)
class TracedClass(object):

    def traced_method(self):
        pass

    # 对于 "_" 开头的方法，默认不会跟踪，若有必要可以使用 trace_private=True
    def _traced_only_if_trace_private_true(self):
         pass

# 添加元类
@six.add_metaclass(profiler.TracedMeta)
class RpcManagerClass(object):
    __trace_args__ = {'name': 'rpc',
                      'info': None,
                      'hide_args': False,
                      'trace_private': False}

     def my_method(self, some_args):
         pass

     def my_method2(self, some_arg1, some_arg2, kw=None, kw2=None)
         pass
```