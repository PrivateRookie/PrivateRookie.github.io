# openstack创建虚拟机源码阅读

---

在openstack中,虚拟机的创建无疑是非常重要的,了解虚拟机创建流程并阅读nova模块关于创建虚拟机的源码对opensatck开发有很很大帮助,本篇文章将以openstack queens版本为基础.讲解创建虚拟机的源码.由于nova模块代码复杂,而且阅读源码所需知识较多,所以侧重于流程逻辑,源码阅读可能不够详尽指出.

## nova模块结构
![nova system architecture][1]

- DB: 用于存储nova服务相关数据的SQL数据库，通常为MySQL
- API：接收HTTP请求的组件,处理请求并通过[oslo.messaging队列][2]或HTTP与其他组件通信
- Scheduler: 决定虚拟机在哪个主机运行
- Network: 管理IP转发,桥接和vlans
- Compute: 管理虚拟机和虚拟机管理器直接的通信
- Conductor: 处理需要协同的请求(创建/重建),作为数据库代理,或者处理对象转换

## 创建虚拟机流程
为了简便,这里省略了认证过程,实际上一个请求发送过来,还要经过认证和鉴权等过程,确保该用户有权限创建虚拟机.
在openstack的wiki中给出创建一个虚拟的workflow,图有点大,但对于每个组件的工作内容写的非常详细
![创建虚拟机流程][3]
我们可以把创建流程分成几个部分
### 处理请求
用户发送一个创建虚拟机的请求,Nova-Api接收到请求后,Nova-Api负责激活扩展插件,检查虚拟机名称,接收注入文件,提取新虚拟机的网络设置,检查配置和镜像等工作.
然后Nova-Api将处理好的参数集以JSON文件通过HTTP请求发送给Nova的Compute－Api.然后向用户发送带有虚拟机预留ID的相应（这时的相应码202,提示虚拟机创建成功,但实际上虚拟机还未真正创建成功)
Compute-Api在接收到请求后将会检查创建政策,进一步检查虚拟机,网络,镜像和配额,然后正式建立虚拟机的配置.
接着在数据库中生成虚拟机的相应记录,而后通过消息队列发送请求让scheduler选择一个host来创建虚拟机.
### 虚拟机调度和资源准备
scheduler接收到消息后根据消息中filters对所有host进行过滤,最后选出某个host,然后跟新数据库,并通过消息队列向被选定的host发送创建虚拟机消息
被选定的host接收到队列的消息后在数据库中更新虚拟机和任务的状态,然后通过nova network-api为虚拟机创建或获取网络.
接着通过nova volume-api为虚拟机创建新的卷,决定虚拟机的块设备映射,并将卷挂载到虚拟机上
此时虚拟机的调度和资源准备都以完成.
### 在hypervisor上运行虚拟机
被选定的host获取镜像,建立块设备映射,最后生成libvirt.xml最后执行hypervisor的spawn()方法,至此,虚拟机已经在host上运行了.最后一步是跟新虚拟机和任务的状态.

## 源码阅读
下面将根据虚拟机创建时调用的模块顺序阅读代码
### Nova-Api
Nova-Api将处理不同类型的请求写成了各种controler类,而处理创建虚拟机的类被称为`ServerControler`
```python
# nova/api/openstack/compute/servers.py
class ServersController(wsgi.Controller):
    
    # 为了简明,省略了很多用于检查的装饰器
    @wsgi.response(202)
    def create(self, req, body):
         """Creates a new server for a given user."""
        context = req.environ['nova.context']
        server_dict = body['server']
        password = self._get_server_admin_password(server_dict)
        name = common.normalize_name(server_dict['name'])
        description = name
        # create方法干的是些从请求中提取并检查参数的脏活
        # 省略...
        try:
            # 依然是构建参数的代码...
            
            # 调用compute_api创建虚拟机
            (instances, resv_id) = self.compute_api.create(context,
                            inst_type,
                            image_uuid,
                            display_name=name,
                            display_description=description,
                            availability_zone=availability_zone,
                            forced_host=host, forced_node=node,
                            metadata=server_dict.get('metadata', {}),
                            admin_password=password,
                            requested_networks=requested_networks,
                            check_server_group_quota=True,
                            supports_multiattach=supports_multiattach,
                            **create_kwargs)
        # 错误处理...
```
### compute-api
```python
# nova/compute/api.py
class API(base.Base):
    def create(self, context, instance_type,
               image_href, kernel_id=None, ramdisk_id=None,
               min_count=None, max_count=None,
               display_name=None, display_description=None,
               key_name=None, key_data=None, security_groups=None,
               availability_zone=None, forced_host=None, forced_node=None,
               user_data=None, metadata=None, injected_files=None,
               admin_password=None, block_device_mapping=None,
               access_ip_v4=None, access_ip_v6=None, requested_networks=None,
               config_drive=None, auto_disk_config=None, scheduler_hints=None,
               legacy_bdm=True, shutdown_terminate=False,
               check_server_group_quota=False, tags=None,
               supports_multiattach=False):
        """准备实例创建工作，然后将实例信息发送至scheduler，
        由scheduler计算host上创建和在DB创建记录。
        """
        # preparation
        
        # 为了简介,所有参数简略为args和kwargs
        self_create_instance(*args, **kwargs)
        
    def _create_instance(self, context, instance_type,
               image_href, kernel_id, ramdisk_id,
               min_count, max_count,
               display_name, display_description,
               key_name, key_data, security_groups,
               availability_zone, user_data, metadata, injected_files,
               admin_password, access_ip_v4, access_ip_v6,
               requested_networks, config_drive,
               block_device_mapping, auto_disk_config, filter_properties,
               reservation_id=None, legacy_bdm=True, shutdown_terminate=False,
               check_server_group_quota=False, tags=None,
               supports_multiattach=False):
        """核查所有参数"""
        # verifying
        pass
        
        # 获取镜像信息
        if image_href:
            # if image_href is provied, get image via glance api
            image_id, boot_meta = self._get_image(context, image_href)
        else:
            # if image_href is not proved, get image metadata from bdm
            image_id = None
            boot_meta = self._get_bdm_image_metadata(
                context, block_device_mapping, legacy_bdm)
        
        # 继续检查参数
        
        # 由于block device mapping有两种版本,为了兼容,需要检查并在必要时转换
         block_device_mapping = self._check_and_transform_bdm(context,
            base_options, instance_type, boot_meta, min_count, max_count,
            block_device_mapping, legacy_bdm)
        
        # go on checking
        
        # 为了支持cell特性,参见cell wiki
        # https://docs.openstack.org/nova/ocata/cells.html
        if CONF.cells.enable:
            # 创建instance模型对象
            # 检查quota
            
            # 调用rpc api将消息发送到队列
            self.compute_task_api.build_instance(*args, **kwargs)
        
        else:
            compute_task_api.schedule_and_build_instances(*args, **kwargs)
        
        
        return instances, reservation_id
```
### 调度和消息传递代码
nova组件之间可以通过rpc api以消息队列通信,而最后的真正执行的任务的类都在`manager.py`文件中定义.这里我们方便理解省略调度代码
```python
# nova/conductor/api.py
class ComputeTaskAPI(object):
    def schedule_and_build_instance(self, *args, **kwargs):
        # very simple method
        # call rpc api only
        self.conductor_compute_rpc_api.schedule_and_build_instance(*args, **kwargs)

# nova/conductor/rpcapi.py
class ComputeTaskAPI(object):
    def schedule_and_build_instance(self, *args, **kwargs):
        # 构建参数和api版本检查
        # 最后将其发送到消息队列
        cctxt.cast(context, 'schedule_and_build_instance', **kwargs)
```
### manager代码
```python
# nova/compute/manager.py
class ComputeManager(object):
    @wrap_exception()
    @reverts_task_state
    @wrap_instance_fault
    def build_and_run_instance(self, *args, **kwargs):
        # 给资源加锁,避免竞争
        @utils.synchronized(instance.uuid)
        def _locked_do_build_and_run_instance(*args, **kwargs):
            with self._build_semaphore:
                try:
                    result = self._do_build_and_run_instance(*args, **kwargs)
                # handle exceptions
                pass
        # 由于创建虚拟机的工作可能会持续很长时间,为了避免进程阻塞
        # 将这个任务分发给某个worker
        utils.spawn_n(_locked_do_build_and_run_instance,
        context, instance, ...)

    def _do_build_and_run_instance(self, *args, **kwargs):
        # 更新虚拟机和任务状态
        # 解码注入文件
        try:
            with timeutils.StopWatch() as timer:
                self._build_and_run_instance(*args)
        # handle exceptions
    def _build_and_run_instance(self, *args, **kwargs):
        # 获取 image ref
        try:
            scheduler_hints = self._get_scheduler_hints(filter_properties, request_spec)
            rt = self._get_resource_tracker()
            with rt.instance_claime(context, instance, node, limits):
                # 获取群组策略和镜像metadata

                # 通过调用_build_resources创建network和volume
                with self._build_resources(*args) as resources:
                    # handle vm and task state
                    # spawn instance on hypervisor
                    with timeuitls.StopWatch() as time:
                        # 通过driver创建xml,然后真正运行虚拟机
                        self.driver.spawn(*args, **kwargs)
        # handle exceptions
```

  [1]: https://docs.openstack.org/nova/ocata/_images/architecture.svg
  [2]: http://docs.openstack.org/developer/oslo.messaging/
  [3]: https://wiki.openstack.org/w/images/a/a9/Curr-run-instance.png