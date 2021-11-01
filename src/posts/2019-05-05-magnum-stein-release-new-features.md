# Magnum Stein Release New Features

原文链接 [Magnum Release Note - Stein](https://docs.openstack.org/releasenotes/magnum/stein.html)

---


## 新特性

### 8.0.0-8
- 新增 Nginx 作为 K8s Ingress 额外控制器. 可以通过标签 `ingress_controller=niginx` 标签指定
- 使用 Node Problem Detector, Draino 和 AutoScaler 支持 k8s 集群自愈，通过 `auto_healing_enabled=off/on` 来开启或关闭
- 集群支持多 DNS 服务器设置。多 DNS 通过 `,` 分割，如 `8.8.8.8,114.114.114.114`

### 8.0.0
- 在 k8s_fedora_atomic 驱动中在主节点部署 kubelet。之前这只能在 calico 插件启动时完成，现在 kubelet 可以在所有情况下部署。 对于监控主节点（如部署 fluentd）或者使用 k8s control-plane 自管理时非常有用
- 添加与 OpenStack Octavia 组件交互的代码
- 新增 `magnum-status upgrade check` 命令。这个命令允许在 Magnum 升级前运行各种检查用于保证安全升级
- 为了获得更好的集群模板版本控制和减轻维护公共集群模板的痛苦，现在支持改变集群模板名称
- 新增 `tiller_enabled=true/false` 标签控制是否在 k8s_fedora_atomic 集群安装 tiller。默认为 false。新增 `tiller_tag` 标签来选择 tiller 版本, 如果未设置，将会选择与 helm 客户端版本匹配的 tiller。tiller 可以通过 `container_infra_prefix` 标签从私有镜像中拉取。添加 `tiller_namespace` 标签来选择将 tiller 安装在哪个命名空间，默认 magnum-tiller。tiller 通过 k8s job 安装，job 所需的 docker 镜像为 docker.io/openstackmagnum/helm-client
- 对于 k8s_fedora_atomic 集群，会将 flannel 作为 cni 插件运行。部署方法来自 flannel 上游文档。 新增 `flannel_cni_tag` 标签控制 cni 插件版本，具体版本见 `quay.io/repository/coreos/flannel-cni`
- 新增 `grafana_tag` 和 `prometheus_tag` 标签控制 k8s_fedora_atomic 集群 grafana 和 prometheus 版本，默认为 5.1.5 和 v1.8.2
- 添加 `heat_container_agent_tag` 标签以允许用户选择 heat-agent 。stein 默认：stein-dev。
- 在 k8s 集群中添加 heat container agent 以支持集群滚动升级
- 安装 metric-server 服务以替换 heapster. metric-server 通过 helm 安装，所以 `tiller_enable` 标签必须设置为 true。为保证兼容性 Heapster 服务忍让可用。
- 添加 `monitoring_enabled` 标签控制是否通过 helm 安装 prometheus-operator 监控套件。新增 `grafana_admin_passwd` 标签设置 grafana 面板密码，默认为 prom_operator
- 在主节点机器创建完成后，将会立即开始创建 k8s 工作节点，而不是等待主节点所有服务创建完成之后再开始，这会显著减少集群创建时间
- 新增 `master_lb_floating_ip_enabled` 标签控制是否为主节点负载均衡分配浮动 IP，这个标签只在启用 `master_lb_enabled` 后才生效。`master_lb_floating_ip_enblaed` 默认值与 `floating_ip_enabled` 标签相同，`floating_ip_enabled` 标签现在只控制是否为所有节点分配浮动 IP
- 现在 k8s cloud-provider-openstack 支持 keystone 认证和鉴权的钩子，通过这个特性，用户可以通过 `keystone_aut-enabled` 来启用 keystone 认证和鉴权
- `ingress_controler` 新增 octavia 选项，新增 `octavia_ingress_controller_tag` 标签控制是部署 [octava-ingress-controller](https://github.com/kubernetes/cloud-provider-openstack/blob/master/docs/using-octavia-ingress-controller.md)
- 使用 ClusterIP 作为默认的 Prometheus 服务类型，因为 NodePort 服务需要额外配置正确的安全组。 k8s 管理员依然可以在集群创建成功后更改
- 密钥不在是必选项，因为用户可以在镜像中预配置其他登录选项
- 新增 k8s 预删除选项以在删除集群之前删除一些云平台资源，目前只有负载均衡服务类型需要删除
- 现在 k8s 的 OpenStack 驱动支持自动伸缩。但目前 Magnum 没有办法让外部消费者控制删除哪个节点。可选方案时直接调用 Heat API，但这显然不是最好解决办法，同时也让 k8s 社区困惑。综上所属，新增 Magnum API: POST <ClusterID>/actions/resize
- 现在为每个主节点组和工作节点组添加了一个服务器组，以提高灵活性。