---
layout: post
title: 【使用 Rust 写 Parser】4. 解析 binlog
categories: [Tech]
tags: [Rust, Parser, MySQL]
date: 2020-08-16
---

本系列前3篇都是围绕如何解析文本文件，这次我们来尝试解析二进制文件。

1. [binlog 介绍](#binlog-介绍)
   1. [设置](#设置)
   2. [常见命令](#常见命令)
   3. [binlog 结构](#binlog-结构)
2. [解析](#解析)
   1. [熟悉工具](#熟悉工具)
   2. [解析 binlog magic number](#解析-binlog-magic-number)
   3. [解析 header](#解析-header)
   4. [解析 format_desc](#解析-format_desc)
3. [使用 boxercrab](#使用-boxercrab)
4. [总结](#总结)



说到二进制文件，因为其不易阅读，给人高深艰涩之感。因为这种原因，unix 的编程哲学也提倡使用文本化协议，方便读写和编辑，
但二进制文件的好处在于有更多的信息密度，因此在网络领域有不少协议通过二进制数据实现。
这次我选择对 MySQL 的 binlog 进行解析。选择它的原因有几点，首先 binlog 文件可以通过启动一个 mysql 镜像开启 binlog
后写入一些数据获得，而其他网络协议则需要抓包，非常不便，而图像文件如 png 等数据展示不便，给我们的验证结果带来非常大的困难；
其次 binlog 的使用频率非常高，尝试解析 binlog 能加深我们对 binlog 的认知；最后是我发现 Rust 还没有基于 nom 实现的 binlog
解析库，率先尝试我觉得是一项挺有意思的工作。
还有一个需要说明的问题是为什么不用 Rust 包一层 libmysql 实现？其实我的考量在于如果依赖 libmysql, 安装这个工具就必须
安装 mysql, 我在使用 Python 的 mysqlclient 和 Rust 的 diesel 时对安装 mysql 依赖都感到非常厌烦，为什么我还要额外依赖一些
东西，而且还不能被 cargo 管理，相比之下 pymysql 和 sqlx 安装就很方便。因此我决定像 sqlx 一样使用纯 Rust 实现。

## binlog 介绍

这里简单介绍 mysql 设置, 常用相关命令和binlog 文件结构。

### 设置

[boxercrab/tests/mysql/conf/myconf.cnf](https://github.com/PrivateRookie/boxercrab/blob/master/tests/mysql/conf/myconf.cnf) 文件是 boxercrab
项目测试的数据库设置，可以提供参考。`log-bin` 和 `server-id` 必不可少。

如果想自己搭建一个环境，可以通过 boxercrab 提供的 docker-compose 文件快速搭建一个。

```bash
git clone https://github.com/PrivateRookie/boxercrab.git
cd boxercrab

docker-compose -f boxercrab/tests/mysql/docker-compose.yml up -d
```

然后你可以就进入容器使用 `mysql`, `mysqlbinlog` 等工具。

### 常见命令

使用 `mysql` 命令连接数据库后可以通过 `show master status` 查看服务器正在写入 binlog 文件名和位置。

```bash
mysql> show master status;
+------------------+----------+--------------+------------------+-------------------+
| File             | Position | Binlog_Do_DB | Binlog_Ignore_DB | Executed_Gtid_Set |
+------------------+----------+--------------+------------------+-------------------+
| mysql_bin.000001 |      154 | default      |                  |                   |
+------------------+----------+--------------+------------------+-------------------+
1 row in set (0.00 sec)
```

`show binlog events` 可以显示当 binlog 的所有事件，可以为我们确定解析结果正确与否提供依据

```bash
mysql> show binlog events;
+------------------+-----+----------------+-----------+-------------+---------------------------------------+
| Log_name         | Pos | Event_type     | Server_id | End_log_pos | Info                                  |
+------------------+-----+----------------+-----------+-------------+---------------------------------------+
| mysql_bin.000001 |   4 | Format_desc    |         1 |         123 | Server ver: 5.7.30-log, Binlog ver: 4 |
| mysql_bin.000001 | 123 | Previous_gtids |         1 |         154 |                                       |
+------------------+-----+----------------+-----------+-------------+---------------------------------------+
2 rows in set (0.00 sec)
```

最后是非常重要的 `mysqlbinlog` 命令，它是 mysql 提供 binlog 解析工具，在从 binlog 恢复数据库时非常有用，它有个 `-H` 选项
可以显示事件对应的 u8 内容，是我们重要参考依据。

```bash
mysqlbinlog -H -vvvvvv /var/lib/mysql/mysql_bin.000001

...
DELIMITER /*!*/;
# at 4
#200816  5:21:35 server id 1  end_log_pos 123 CRC32 0x51e1fdcb
# Position  Timestamp   Type   Master ID        Size      Master Pos    Flags
#        4 5f c2 38 5f   0f   01 00 00 00   77 00 00 00   7b 00 00 00   01 00
#       17 04 00 35 2e 37 2e 33 30  2d 6c 6f 67 00 00 00 00 |..5.7.30.log....|
#       27 00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00 |................|
#       37 00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00 |................|
#       47 00 00 00 00 5f c2 38 5f  13 38 0d 00 08 00 12 00 |......8..8......|
#       57 04 04 04 04 12 00 00 5f  00 04 1a 08 00 00 00 08 |................|
#       67 08 08 02 00 00 00 0a 0a  0a 2a 2a 00 12 34 00 01 |.............4..|
#       77 cb fd e1 51                                      |...Q|
#       Start: binlog v 4, server v 5.7.30-log created 200816  5:21:35 at startup
# Warning: this binlog is either in use or was not closed properly.

```

### binlog 结构

每个 binlog 都以 [fe 'bin' ] 开头，即 `[254, 98, 105, 110]`。接着是各种 binlog 事件。每个事件结构分为 header 和 payload，header 结构如下（数字代表使用几个字节）

```
4              timestamp
1              event type
4              server-id
4              event-size
   if binlog-version > 1:
4              log pos
2              flags
```

可见 header 声明了一个事件的事件，类型，写入 binlog 的服务器ID，下个事件在 binlog 的位置和 flag。需要注意的是 binlog 采用小端序列，也就说 server-id 为1 的u32 应该为 `0x01 0x00 0x00 0x00`。

payload 则因为事件不同结构各有差异，但表示方法与 header 相同。每种事件的 payload 结构可以在 [Binlog Event Type](https://dev.mysql.com/doc/internals/en/binlog-event-type.html) 查看，或者在 mysql-server 源码中也有相应的注释。不过
因为 mysql 文档和源码中的 payload, post header, body 等词语经常混用，阅读应该主要甄别。

如果你嫌麻烦，我在写 boxercrab 时在对应的 struct 都加上了每个事件的参考连接，使用 vscode 可以按住 ctrl 然后鼠标点击连接即可跳转 [boxercrab/src/events/mod.rs](https://github.com/PrivateRookie/boxercrab/blob/cbbd5119d8478fade5e52d400f5215ab7f93e816/src/events/mod.rs#L87-L377)。

紧跟着 `[254, 98, 105, 110]` 的第一个事件是 format_desc 或 start_event_v3。
format_desc 为新版本(v4) 使用。boxercrab 只支持 format_desc。

format_desc 声明了 binlog 版本、服务器版本和 header 长度，是解析 event 的重要依据。下面我们将先尝试 format_desc 事件。


## 解析

### 熟悉工具

nom 文档没有对如何解析二进制数据进行详细的，不过好在 nom 对每个函数都有详细的文档和使用说明，doc.rs 还提供了搜索，我们可以方便地进行查阅 [nom doc.rs](https://docs.rs/nom/5.1.2/nom/)

在解析 binlog 时常有的函数有将数据转换为 usize 或 int, nom 提供了大端和小端对应的解析函数， 如 `le_u16` 可以将输入解析为 `u16` 类型；
另外就是常见的组合子，如 `map`, `take` 和 `tag` 等，这些组合子用法与解析文本格式时的用法类似。如果你还不太熟悉，可以先翻阅下 nom 的文档。

### 解析 binlog magic number

按照之前的描述，所有 binlog 文件都以特定的 `.bin` 开头，解析时我们要首先检查4个magic number, 确认这是个 binlog 文件。

与文本解析类似，对于二进制文件，你也可以使用 `tag`，因此我们的检查函数可以写为

```rust
pub fn check_start(i: &[u8]) -> IResult<&[u8], &[u8]> {
    tag([254, 98, 105, 110])(i)
}
```

可以看到我们的返回值类型从 `&str` 变成 `&[u8]` 但 nom 仍然可以工作。

### 解析 header

每个事件都有 header, 解析事件之前我们必须解析 header，同时 header 结构也非常简单，适合热手。我们可以通过 `mysqlbinlog` 工具查看每个事件的 header, 在添加了`-H`选线时，binlog header 会被放在每个事件的第一行。

```bash
#200731  6:07:14 server id 1  end_log_pos 123 CRC32 0x5b1860c0
# Position  Timestamp   Type   Master ID        Size      Master Pos    Flags
#        4 12 b5 23 5f   0f   01 00 00 00   77 00 00 00   7b 00 00 00   00 00
```

按照之前的描述的 header 结构，我们首先要读取一个4字节的 timestamp, 就是 `u32`, 接着是1个代表事件类型的 `u8`, 而后是 `u32` 的 server_id, `u32` 事件大小，`u32` log position(我们只解析v4版本，所以log position一定存在) 和 `u16` flag。

nom 提供了 `le_u32`, `le_u8`, `le_u16` 等小端整数解析函数，因此
按照这个描述，header 解析函数可以写为

```bash
pub fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let (i, timestamp) = le_u32(input)?;
    let (i, event_type) = le_u8(i)?;
    let (i, server_id) = le_u32(i)?;
    let (i, event_size) = le_u32(i)?;
    let (i, log_pos) = le_u32(i)?;
    let (i, flags) = map(le_u16, |f: u16| EventFlag {
        in_use: (f >> 0) % 2 == 1,
        forced_rotate: (f >> 1) % 2 == 1,
        thread_specific: (f >> 2) % 2 == 1,
        suppress_use: (f >> 3) % 2 == 1,
        update_table_map_version: (f >> 4) % 2 == 1,
        artificial: (f >> 5) % 2 == 1,
        relay_log: (f >> 6) % 2 == 1,
        ignorable: (f >> 7) % 2 == 1,
        no_filter: (f >> 8) % 2 == 1,
        mts_isolate: (f >> 9) % 2 == 1,
    })(i)?;
    Ok((
        i,
        Header {
            timestamp,
            event_type,
            server_id,
            event_size,
            log_pos,
            flags,
        },
    ))
}
```

可以看到从 timestamp 到 log_pos 都只是简单地调用 nom 提供的解析函数，写法非常简单。而 flag 解析则相对复杂点，它先被解析为一个 `u16`，但它实际上是一个 bit_flag, 所以我们根据 [Binlog Event Flag](https://dev.mysql.com/doc/internals/en/binlog-event-flag.html) 对 `u16` 进行右移得到真正的 flag。

解析 binlog 其实没有那么难对嘛，下面我们开始解析 format_desc 事件的 payload。

### 解析 format_desc

用 `mysqlbinlog` 查看 format_desc 对应的 hexdump 如下

```bash
#200731  6:07:14 server id 1  end_log_pos 123 CRC32 0x5b1860c0
# Position  Timestamp   Type   Master ID        Size      Master Pos    Flags
#        4 12 b5 23 5f   0f   01 00 00 00   77 00 00 00   7b 00 00 00   00 00
#       17 04 00 35 2e 37 2e 33 30  2d 6c 6f 67 00 00 00 00 |..5.7.30.log....|
#       27 00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00 |................|
#       37 00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00 |................|
#       47 00 00 00 00 12 b5 23 5f  13 38 0d 00 08 00 12 00 |.........8......|
#       57 04 04 04 04 12 00 00 5f  00 04 1a 08 00 00 00 08 |................|
#       67 08 08 02 00 00 00 0a 0a  0a 2a 2a 00 12 34 00 01 |.............4..|
#       77 c0 60 18 5b                                      |....|
# 	Start: binlog v 4, server v 5.7.30-log created 200731  6:07:14 at startup
ROLLBACK/*!*/;
BINLOG '
ErUjXw8BAAAAdwAAAHsAAAAAAAQANS43LjMwLWxvZwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
AAAAAAAAAAAAAAAAAAAStSNfEzgNAAgAEgAEBAQEEgAAXwAEGggAAAAICAgCAAAACgoKKioAEjQA
AcBgGFs=
'/*!*/;
```

为了方便，我们把所有 event 都放到一个 enum 下，这样，在写解析函数时不用操心返回值类型，缺点时这会导致 enum 定义非常长: (。具体定义可以在 boxercrab 项目源码看到，这些不再贴出来。

format_desc 全称为 format_description_event，它的 payload 结构为(mysql 文档中遗漏了 checksum_alg 和 checksum 字段)

```
2                binlog-version
string[50]       mysql-server version
4                create timestamp
1                event header length
string[p]        event type header lengths
1                checksum alg
4                checksum
```

首先是一个 `u16` 代表的 binlog 版本，接着是一个固定长度为 50 的字符串（可能包含多个 \0 终止符)，然后是 `u32` 的 timestamp 和一个一直到事件结尾(去除后面 checksum 和算法)的数组，代表事件 header 长度。

boxercrab 中的实现为

```rust
fn parse_format_desc<'a>(input: &'a [u8], header: Header) -> IResult<&'a [u8], Event> {
    let (i, binlog_version) = le_u16(input)?;
    let (i, mysql_server_version) = map(take(50usize), |s: &[u8]| extract_string(s))(i)?;
    let (i, create_timestamp) = le_u32(i)?;
    let (i, event_header_length) = le_u8(i)?;
    let num = header.event_size - 19 - (2 + 50 + 4 + 1) - 1 - 4;
    let (i, supported_types) = map(take(num), |s: &[u8]| s.to_vec())(i)?;
    let (i, checksum_alg) = le_u8(i)?;
    let (i, checksum) = le_u32(i)?;
    Ok((
        i,
        Event::FormatDesc {
            header,
            binlog_version,
            mysql_server_version,
            create_timestamp,
            event_header_length,
            supported_types,
            checksum_alg,
            checksum,
        },
    ))
}
```

可以看到我们都是按照协议的描述一个字段一个字段地解析，甚至 if else 都没用上。
在解析 mysql_server_version 时我们先拿出50字节，然后试着将这50字节转换为字符串。
由于可能存在多个终止符，我们首先需要找到第一个终止符位置，然后使用 `String::from_utf8_lossy` 将之前的字符转换为字符串。这就是 `extract_string` 的实现思路

```rust
/// extract n(n <= len(input)) bytes string
pub fn extract_string(input: &[u8]) -> String {
    let null_end = input
        .iter()
        .position(|&c| c == b'\0')
        .unwrap_or(input.len());
    String::from_utf8_lossy(&input[0..null_end]).to_string()
}
```

对于 supported_types 要取多少个字节，我们可以用 header 拿到的 event_size 减去 header 大小，和其他字段大小，剩下的自然就是 supported_types 占用的字节数，所以才会有 `header.event_size - 19 - (2 + 50 + 4 + 1) - 1 - 4`。

最后我们把 parse_header 和 parse_format_desc 结合起来，

```rust
pub fn parse<'a>(input: &'a [u8]) -> IResult<&'a [u8], Event> {
    let (input, header) = parse_header(input)?;
    match header.event_type {
        0x0f => parse_format_desc(input, header),
        ... // 省略其他事件类型
        t @ _ => {
            log::error!("unexpected event type: {:x}", t);
            unreachable!();
        }
    }
```

这就结束了吗？是的，这就结束了，解析 binlog 就是这么简单，解析事件都是按照这个思路来就行了。
没有那么多弯弯绕绕。那为什么我为了完整解析 binlog 用了快1个月呢，因为不论是 MySQL 文档还是源码，它们的说明都是残缺的，某些事件字段完全对不上，我不得不通过对比 mysqlbinlog 工具解析出来的内容来实现对应的解析函数，这才是耗费时间的大头。

如果你对其他事件的结构或解析感兴趣可以查看 [boxercrab/src/events/mod.rs](https://github.com/PrivateRookie/boxercrab/blob/master/src/events/mod.rs) 对应的解析实现。

解析二进制文件相对简单，以至于某些作者在 nom 的基础上使用过程宏让你可以通过“指令”即可完成解析，不用写额外的解析函数，他就是 [nom_derive](https://docs.rs/nom-derive/0.6.0/nom_derive/derive.Nom.html)。感性趣的朋友可以自己查看文档。

## 使用 boxercrab

最后为我的项目 boxercrab 打个广告，它原本只是练手项目，但我在阅读了 canal 的源码和文档我觉得可以把它变成类似于 canal 的项目。目前项目已经实现了绝大部分事件类型的解析，
而且不少事件都有测试代码，核心功能基本完成。目前 boxercrab 还提供了命令行 [bcrab](https://github.com/PrivateRookie/boxercrab/blob/master/src/cli.rs)
示范如何使用 boxercrab。
现在你可以通过 `bcrab` 将 binlog 转换为一个 json 或 yaml 文件(trans 命令)，方便
各位学习 binlog 事件的结构；或者通过 conn 命令直接连接都一台 mysql-server, 将自己伪装成 client, 持续监听 binlog 文件，并将解析后事件打印到屏幕上。

具体使用方法可以查看项目 [README.md](https://github.com/PrivateRookie/boxercrab/blob/master/README.md)

因为对 canal 不熟悉，要为 boxercrab 添加哪些功能是我很头疼的问题，如果各位有好的意见或建议，又或者在使用时遇到了问题，欢迎在提 Issue 给我提供思路和 usage case。

## 总结

nom 确实是个好框架，文档详细，用着方便，性能不错: )

至此使用 nom 写 parser 系列基本完结，原本计划写的实现一门语言因精力有限可能会拖到很久才写完，但从第1篇到第4篇，我们实现了普通文本和二进制文件解析，从简单的 rgb 解析，到比较复杂的 redis 和 json 解析，到最后写成一个独立项目的 boxercrab，我个人收获挺多的，也希望这系列能给读者一些帮助🎉

---

以下是我在实现过程中的参考资料或项目

1. 如何判断 binlog 版本 https://dev.mysql.com/doc/internals/en/determining-the-binlog-version.html
2. A binlog file starts with a Binlog File Header [ fe 'bin' ] https://dev.mysql.com/doc/internals/en/binlog-file-header.html#:~:text=A%20binlog%20file,Header%20%5B%20fe%20%27bin%27%20%5D
3. Binlog Event header https://dev.mysql.com/doc/internals/en/binlog-event-header.html
4. Binlog Event Type https://dev.mysql.com/doc/internals/en/binlog-event-type.html
5. Binlog Event Flag https://dev.mysql.com/doc/internals/en/binlog-event-flag.html
6. nom derive 0.6: deriving binary parsers from structure declaration : rust https://www.reddit.com/r/rust/comments/hdb5h1/nom_derive_06_deriving_binary_parsers_from/
7. ospf-parser/ospfv3.rs at master · rusticata/ospf-parser https://github.com/rusticata/ospf-parser/blob/master/src/ospfv3.rs
8. FORMAT_DESCRIPTION_EVENT - MariaDB Knowledge Base https://mariadb.com/kb/en/format_description_event
9. mysqlbinlog2/mysqlbinlog.cpp at master · harubot/mysqlbinlog2 https://github.com/harubot/mysqlbinlog2/blob/master/mysqlbinlog.cpp
10. shyiko/mysql-binlog-connector-java: MySQL Binary Log connector
11. https://github.com/shyiko/mysql-binlog-connector-java
12. noplay/python-mysql-replication: Pure Python Implementation of MySQL replication protocol build on top of PyMYSQL https://github.com/noplay/python-mysql-replication
13. MySQL :: MySQL 5.6 Reference Manual :: 17.1.4.4 Binary Log Options and Variables https://dev.mysql.com/doc/refman/5.6/en/replication-options-binary-log.html#sysvar_binlog_row_image
14. binlog - GoDoc https://godoc.org/github.com/dropbox/godropbox/database/binlog#BaseRowsEvent
15. MySQL :: MySQL Internals Manual :: 20.9 Event Data for Specific Event Types https://dev.mysql.com/doc/internals/en/event-data-for-specific-event-types.html
16. mysql-server/rows_event.h at 91a17cedb1ee880fe7915fb14cfd74c04e8d6588 · mysql/mysql-server https://github.com/mysql/mysql-server/blob/91a17cedb1/libbinlogevents/include/rows_event.h
17. MySQL :: MySQL 5.7 Reference Manual :: 16.1.3.1 GTID Format and Storage https://dev.mysql.com/doc/refman/5.7/en/replication-gtids-concepts.html
18. MySQL 5.6 GTID 原理以及使用 - 泽锦 - 博客园 https://www.cnblogs.com/zejin2008/p/7705473.html
19. MySQL :: MySQL 5.7 Reference Manual :: 12.22.2 DECIMAL Data Type Characteristics https://dev.mysql.com/doc/refman/5.7/en/precision-math-decimal-characteristics.html
20. MySQL :: MySQL 8.0 Reference Manual :: 11 Data Types https://dev.mysql.com/doc/refman/8.0/en/data-types.html


https://zhuanlan.zhihu.com/p/115017849
https://zhuanlan.zhihu.com/p/139387293
https://zhuanlan.zhihu.com/p/146455601
https://zhuanlan.zhihu.com/p/186217695