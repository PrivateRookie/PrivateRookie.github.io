# 使用 screen 让 gitlab-runner 运行后台任务

---

今天在为项目配置 gitlab CI 时需要在 script 中启动一个后台任务，在脚本中
可以用 `nohup` 和 `&` 让 shell 在后台运行任务。可是当 gitlab-runner 运行脚本时，
gitlab-runner 只有在所有子进程成功退出才会退出此任务， 因此如果有后台任务，此次 CI 任务将会一直挂着。
经过很多尝试，依然没有办法（我没有服务器的 sudo 权限， 因此不能通过 `systemd` 等运行 daemon 进程）。
好在可以用 `screen` 实现，而且实现方法相当简单。

```bash
screen -dmS <session_name> bash -c "command to run"
```

`screen -dmS` 可以启动一个自动脱离的且命名的会话，接着是我们希望在会话中执行的命名，这里我们用 `bash -c` 保证命令在 bash shell 环境运行

gitlab CI 配置文件可以这样写

```yml
run_server:
    stage: deploy
    script:
        - bash launch.sh
```

`launch.sh` 中启动一个 screen

```bash
# NOTE 我们需要先 kill 掉之前的进程以关闭会话
ps aux | grep <pattern> | awk {' print $2 '} | xargs kill

screen -dmS <session_name> bash -c <command_to_run>
```

因为 screen 会自动脱离且保持 shell 不中断，CI 任务将会退出，同时又可以运行“后台”任务。