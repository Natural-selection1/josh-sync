# 最终目标: 自动双向同步

对于自动化同步而言, 因为最终需要创建仓库上的PR, 所以应当总是由需要同步的一方主动pull.

与此同时, 存在 monorepo 和 subrepo 角色上的区分.
是 `1 : n` 的关系

```txt
subrepoA <--pull-- monorepo

monorepo <--pull-- subrepoX
monorepo <--pull-- subrepoY
monorepo <--pull-- subrepoZ
```

# sync baseline 设计

同步的最根本原则是`不能错过`, 即:

> 这一次的同步范围的起点 必须不晚于 上一次的同步范围的终点

因此选择一个较老的固定基线必然不会出错,
本设计旨在探索一个尽可能精确的同步范围.

## subrepo <--pull-- monorepo

> 需求: 知道该从 monorepo 的哪里开始同步到最新.

由于 subrepo 并不知道 monorepo 的任何信息.

所以每个 subrepo 都需要 sync-version 文件,
用于记录最近一次同步 monorepo 时, monorepo 的 git SHA.

(初始化时该文件为空)
(该文件禁止任何人为修改)

同步范围就是 `sync-version..=LATEST`,
并且在同步PR中更新 sync-version 为 `LATEST`

下一次同步的范围就是 `sync-version(上一次的LATEST)..=LATEST`

## monorepo <--pull-- subrepo

> 需求: 知道该从 subrepo 的哪里开始同步到最新.

monorepo 拥有 subrepo 最近一次同步时的快照.

### 使用 subrepo 的快照本身?

由于 monorepo 也能够对该快照进行修改(跨仓库修改正是我们的使用josh的原因),
因此该快照本身不能作为同步baseline.

### 使用快照里的sync-version?

#### 定义

对某一个固定的 subrepo 和固定的 Josh filter，设：

- `Eᵢ`：上一次成功的 `monorepo <--pull-- subrepo` 实际纳入的
  subrepo 末尾提交。
- `Vᵢ`：该次同步从 `Eᵢ` 带入 monorepo 快照的 `sync-version` 值。
- `Pᵢ`：subrepo 中把 `sync-version` 写为 `Vᵢ` 的 preparation commit。
- `Rᵢ`：产生 `Pᵢ` 的那次 `subrepo <--pull-- monorepo` 的完整同步结果。
  `Rᵢ` 既包含 `Pᵢ`，也包含从 monorepo 拉取的 Josh 映射历史。
- `Nᵢ`：本次 `monorepo <--pull-- subrepo` 准备拉取的 subrepo 末尾提交。
- `F(X)`：monorepo SHA 为 `X` 时，由当前固定的 Josh filter 映射得到的
  subrepo SHA。
- `A ⪯ B`：`A` 是 `B` 的祖先或等于 `B`。

不能把 `Pᵢ` 和 `Rᵢ` 视为同一个提交。当前 josh-sync 会先创建
`Pᵢ`，然后才 merge `F(Vᵢ)`。因此一般不能证明
`F(Vᵢ) ⪯ Pᵢ`；能够证明的是 `F(Vᵢ) ⪯ Rᵢ`。

#### 需要维持的不变量

1. `sync-version` 只能由 `subrepo <--pull-- monorepo` 流程写入，
   不能手工修改或从其他历史 cherry-pick。
2. 写入 `sync-version = Vᵢ` 和纳入 `F(Vᵢ)` 必须是一个不可分割的同步结果。
   只有包含完整 `Rᵢ` 的 PR 才能进入 subrepo 主分支；不能只纳入 `Pᵢ`。
3. 同步提交不能被 squash、rebase 或以其他方式改写历史，
   否则 `F(Vᵢ)` 的祖先关系会丢失。
4. 从 `Eᵢ` 到 `Nᵢ` 的 subrepo 主分支没有被 force-push 到无关历史，
   即 `Eᵢ ⪯ Nᵢ`。
5. 在这个证明适用期间，Josh filter、filter 版本和映射路径保持不变。
   换言之，写入 `Vᵢ` 时使用的 `F` 和当前计算 baseline 时使用的 `F`
   必须是同一个函数。
6. `monorepo <--pull-- subrepo` 和后续的 monorepo 修改必须保留从
   `Eᵢ` 带入的 `sync-version`，不能在 monorepo 中直接改写它。

#### 证明

1. `Vᵢ` 来自 `Eᵢ` 中的 `sync-version`。根据不变量 1 和 2，
   它必然由某次已成功进入 subrepo 主分支的完整同步结果 `Rᵢ`
   引入，而不是由一个孤立的 preparation commit 引入。所以：

   ```text
   Rᵢ ⪯ Eᵢ
   ```

2. 该次 `subrepo <--pull-- monorepo` 写入的值是 `Vᵢ`，所以它拉取的
   monorepo 末尾也是 `Vᵢ`。完整同步结果 `Rᵢ` 包含 Josh 映射提交
   `F(Vᵢ)`，所以：

   ```text
   F(Vᵢ) ⪯ Rᵢ
   ```

3. 由祖先关系的传递性：

   ```text
   F(Vᵢ) ⪯ Rᵢ ⪯ Eᵢ
   ```

   因此：

   ```text
   F(Vᵢ) ⪯ Eᵢ
   ```

4. 如果 subrepo 主分支没有改写历史，则 `Eᵢ ⪯ Nᵢ`。再次使用传递性：

   ```text
   F(Vᵢ) ⪯ Eᵢ ⪯ Nᵢ
   ```

所以 `F(Vᵢ)` 不晚于上一次 `monorepo <--pull-- subrepo` 的同步末尾
`Eᵢ`，并且是本次 subrepo 末尾 `Nᵢ` 的祖先。在上述不变量成立时，
**`F(Vᵢ)` 可以作为本次 `monorepo <--pull-- subrepo` 的安全 baseline，
不会漏掉上一次同步之后的提交。**

这个结论只保证安全性，不保证 baseline 精确。`F(Vᵢ)` 可能严格早于
`Eᵢ`；如果 subrepo 长期只向 monorepo 输出修改，`Vᵢ` 没有前进，
这个 baseline 就会逐渐变旧，但仍然不会漏提交。

这里的“安全”仅指同步范围的覆盖性：从 `F(Vᵢ)` 开始不会跳过
`Eᵢ` 之后的提交。它不单独证明较旧范围中的提交被重新处理时一定无冲突，
也不单独证明同步后的 tree 正确。这两点还需要由 Josh 的历史保留性质、
merge 算法和 round-trip check 分别保证。

实现不应当只依赖上述逻辑推导。在开始同步前，应当 fetch `F(Vᵢ)` 和
`Nᵢ`，并使用等价于以下命令的检查验证证明的最终结论：

```bash
git merge-base --is-ancestor F(Vᵢ) Nᵢ
```

如果检查失败，必须停止同步并要求重新 bootstrap，不能退化为一个
未经证明的 baseline。

#### 不适用的情况

- 初始化时 `sync-version` 为空，没有 `V` 可以用于上述证明，
  必须从一对已验证等价的 monorepo/subrepo 提交 bootstrap。
- filter、filter 版本或映射路径发生变化时，新的 `F(V)` 不一定仍在
  subrepo 的历史中，必须重新 bootstrap 或进行专门的 baseline 迁移。
- 任意一个不变量被破坏时，不能继续信任 `F(V)`。同步程序应当拒绝同步，
  而不是猜测一个 baseline。

# 回声分析

创建同步PR的要求是: SHA != sync-version && 存在非空diff

1. monorepo 的真实改动
2. subrepo 同步：真实改动 + 更新 sync-version
3. monorepo 回同步：(通常只更新刚刚subrepo更新的 sync-version)
4. subrepo 再同步：虽然 SHA 和 sync-version不同, 但 Josh 内容没有新增变化,
   回滚本次 sync-version 预提交，不创建 PR.

# 安全检查

round-trip 该在哪里做?

# cli设计

`josh-sync init`
`josh-sync gen`
`josh-sync pull`
`josh-sync push` (待定, 或许可以保留现有的行为?)
`josh-sync help`

## `josh-sync init`

生成 `josh-sync.toml`, 以及可能存在的 sync-version (仅 --role subrepo 时)

(文件中是填充好参考内容和注释的)
(同时还要支持 通过cli初始化时就填充好数据)

## `josh-sync gen`

根据已经存在的 toml 配置文件生成 CI `*.yml` 文件
(同时还要支持 通过cli初始化时就填充好数据)

## `josh-sync pull`

从别人那里获取同步, 需要根据 josh-sync.toml 中的 role 进行行为上的细分

任意local, fork, upstream都应该可以从任意local, fork, upstream拉取

## `josh-sync push`

可以沿用现在的实现?

## `josh-sync help`

由 clap 管理

# josh-sync.toml 设计

monorepo 还是 subrepo，都使用 `josh-sync.toml` 作为配置，
并使用同一套字段和同一个解析入口。`role` 只用于决定配置约束和
`pull` 时的同步方向，不维护两套独立的配置 schema。

现阶段所有列出的字段都是必填项：不使用默认值，也不允许根据
Git remote 或其他环境信息隐式推导缺失字段。解析可以先产生统一的
raw config，验证后再转换为 `MonorepoConfig` 或 `SubrepoConfig`。

- 对于 subrepo 而言, 其需要一个monorepo的声明, 以及与仓库本身映射相关的配置
- 对于 monorepo 而言, 其需要一个subrepo声明数组, 以及与仓库本身映射相关的配置

`filter` 是映射的唯一事实来源。它始终站在 monorepo 的视角编写：
输入 monorepo 历史，输出 subrepo 根目录下应该看到的过滤历史。

```text
filter(monorepo) => subrepo view
```

两个仓库对同一映射必须使用完全相同的 canonical filter。
`subrepo <--pull-- monorepo` 直接应用该 filter；
`monorepo <--pull-- subrepo` 必须使用 Josh 的 reverse 能力将它反向应用，
不由用户手工编写第二个反向 filter。

role 相关约束：

- `role = "subrepo"` 时，
  `[[subrepo]]` 必须恰好只有一项，且该项就是当前仓库。
- `role = "monorepo"` 时，
  `[[subrepo]]` 必须至少有一项，每项表示一个可独立同步的 subrepo。
- `role` 不能被 `pull` 的命令行参数覆盖。

字段验证约束：

- `filter-version` 必须是工具支持的明确版本。
- 所有 `git-url`、`git-forge`、`owner`、`name` 和 `sync-branch`
  都必须非空；`owner/name` 必须与 `git-url` 及 `git-forge` 一致。
- `filter` 必须非空，并且能被当前 `filter-version` 对应的 Josh
  正向解析、反向应用和 round-trip。
- 同一 monorepo 中的 subrepo `owner/name` 不能重复，规范化后完全
  相同的 filter 也不能重复。
- 任意 Josh filter 的写入范围可能重叠，这不能再通过简单的路径前缀
  检查完全判定。monorepo 的多 subrepo 同步默认应串行执行，
  并在候选结果上检查实际写入范围。

`sync-version` 的文件名和在 subrepo view 中的位置不可配置，
而是协议的一部分：

```text
subrepo view 中: <repo-root>/sync-version
```

对于 subrepo，这就是工作树根目录下的 `sync-version`。对于
monorepo，不假定它在原始树中对应某个可直接推导的物理路径；
工具应当对 monorepo commit 应用 canonical filter，然后从产生的
subrepo view 根目录读取它。

以上字段足以完成同步引擎的核心功能：确定两个仓库、源和目标分支、
双向历史映射、`sync-version` 位置，以及创建 PR 时所需的 forge
地址和仓库名。

`josh-sync gen` 的输入边界仍需要单独确定。定时表达式、GitHub App ID、
private-key secret 名、PR 作者和 josh-sync revision 不属于上述映射信息。
如果 `gen` 的目标是无额外输入生成可直接运行的 CI，这些项也必须作为
配置字段显式写入；否则 `gen` 必须把它们定义为必填命令行参数，
不能使用隐式默认值。

```toml
# for subrepo apps_shell
role = "subrepo"
filter-version = 2

[monorepo]
git-url = "https://github.com/vivoblueos-lab/blueos"
git-forge = "https://github.com"
owner = "vivoblueos-lab"
name = "blueos"
sync-branch = "main"

[[subrepo]]
git-url = "https://github.com/vivoblueos-lab/apps_shell"
git-forge = "https://github.com"
owner = "vivoblueos-lab"
name = "apps_shell"
# 始终站在 monorepo 的视角编写：
# 对 monorepo 应用该 filter 后，应得到 subrepo 根目录下的内容。
filter = ":/apps/shell"
sync-branch = "main"
```

```toml
# for monorepo blueos
role = "monorepo"
filter-version = 2

[monorepo]
git-url = "https://github.com/vivoblueos-lab/blueos"
git-forge = "https://github.com"
owner = "vivoblueos-lab"
name = "blueos"
sync-branch = "main"

[[subrepo]]
git-url = "https://github.com/vivoblueos-lab/apps_shell"
git-forge = "https://github.com"
owner = "vivoblueos-lab"
name = "apps_shell"
# 始终站在 monorepo 的视角编写：
# 对 monorepo 应用该 filter 后，应得到 subrepo 根目录下的内容。
filter = ":/apps/shell"
sync-branch = "main"

[[subrepo]]
git-url = "https://github.com/vivoblueos-lab/libc"
git-forge = "https://github.com"
owner = "vivoblueos-lab"
name = "libc"
# 始终站在 monorepo 的视角编写：
# 对 monorepo 应用该 filter 后，应得到 subrepo 根目录下的内容。
filter = ":/libc"
sync-branch = "blueos-dev"
```
