# CC Switch CLI 功能实现评估

> 文档日期：2026-06-27 ｜ 最后更新：2026-06-27 ｜ 状态：三阶段全部完成
>
> 目的：对 CLI 与 GUI 的功能差异逐项评估，分为"必须实现"、"可实现"、"没必要实现"三类，并为每项功能编号。
>
> **核心原则**：GUI 的主体功能是代理和协议转换，其他均为附带功能。CLI 必须实现的核心是代理运行与协议转换的等价能力；MCP/Prompt/Skills/环境变量/会话等属附带功能列为可实现；云同步/AUTH/Keychain 等非核心功能列为没必要实现。
>
> **实现状态**：Phase 1（REQ-001~009）✅、Phase 2（REQ-010~019）✅、Phase 3（OPT 系列关键功能）✅ 已全部实现。完整命令参考见 `docs/cli-reference-manual.md`。

---

## 一、评估原则

**必须实现**：围绕代理服务和协议转换的核心功能。代理是 GUI 的主体功能，CLI 必须能完整管理代理生命周期、供应商配置、故障转移、请求处理、用量监控等，使 CLI 达到与 GUI 等价的代理运行能力。

**可实现**：MCP、Prompt、Skills、环境变量、会话管理等附带功能。这些功能有价值但非代理核心流程，可根据用户需求择机实现。

**没必要实现**：云同步、OAuth 认证、Keychain 以及强依赖 GUI 运行时（窗口、托盘、系统对话框、浏览器交互）的功能。这些在无头环境下要么无意义，要么可用系统级方案替代。

---

## 二、必须实现（代理与协议转换核心）

### 供应商管理（代理的请求目标）

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| REQ-001 | 供应商排序 | 供应商顺序影响故障转移优先级，代理核心配置 | `update_providers_sort_order` |
| REQ-002 | 从 Live 配置导入 | 代理首次部署必须能从现有 Claude/Codex/Gemini 等配置文件导入供应商 | `import_default_config` / `import_opencode_providers_from_live` / `import_openclaw_providers_from_live` / `import_hermes_providers_from_live` |
| REQ-003 | 读取 Live 配置 | 查看当前 live 配置文件实际内容，排查代理配置问题必备 | `read_live_provider_settings` |
| REQ-004 | 模型列表获取 | 协议转换需要知道供应商支持的模型列表，否则转换无法正确映射 | `fetch_models_for_config` |
| REQ-005 | 同步到 Live | 将数据库供应商配置写回 live 配置文件，代理接管和手动同步都需要 | `sync_current_providers_live` |

### 代理配置与运行管理

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| REQ-006 | 代理配置读写 | 代理监听地址、端口等参数配置，`start` 命令的环境变量方式不够完整 | `get_proxy_config` / `update_proxy_config` / `get_global_proxy_config` / `update_global_proxy_config` / `get_proxy_config_for_app` / `update_proxy_config_for_app` |
| REQ-007 | 成本倍率设置 | 代理成本统计的基础配置，影响用量统计准确性 | `get_default_cost_multiplier` / `set_default_cost_multiplier` |
| REQ-008 | 计费模型来源 | 切换计费模型来源影响协议转换后的成本统计 | `get_pricing_model_source` / `set_pricing_model_source` |
| REQ-009 | Live 接管检测 | 检测 live 配置是否被代理接管，排查代理状态必备 | `is_live_takeover_active` |

### 故障转移与熔断器（代理可靠性）

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| REQ-010 | 熔断器统计 | 代理运行必须能监控熔断器状态，判断故障转移是否正常工作 | `get_circuit_breaker_stats` |
| REQ-011 | 供应商健康状态 | 代理故障转移决策依据，必须能查看供应商可用性 | `get_provider_health` |
| REQ-012 | 可用故障转移列表 | 配置故障转移队列时需要知道哪些供应商可加入 | `get_available_providers_for_failover` |

### 请求处理配置（协议转换）

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| REQ-013 | 通用配置片段管理 | 多供应商共享配置（公共环境变量、通用请求头等），协议转换的公共部分 | `get_common_config_snippet` / `set_common_config_snippet` / `extract_common_config_snippet` |

### 代理用量监控（代理产生的数据）

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| REQ-014 | 按应用统计用量 | 代理请求按应用分类统计，监控代理流量分布 | `get_usage_summary_by_app` |
| REQ-015 | 请求日志查看 | 代理产生的请求日志，排查请求失败和协议转换问题必备 | `get_request_logs` / `get_request_detail` |
| REQ-016 | 供应商限额检查 | 监控代理转发的用量是否接近供应商限额，避免超额 | `check_provider_limits` |

### 配置与备份管理（代理配置的持久化）

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| REQ-017 | 备份删除 | 代理配置备份管理基本功能，当前 CLI 只能创建和恢复 | `delete_db_backup` |
| REQ-018 | 备份重命名 | 标记重要备份点，便于回滚代理配置 | `rename_db_backup` |
| REQ-019 | 自定义端点管理 | 代理测速端点的增删改，`speedtest` 命令的配套功能 | `get_custom_endpoints` / `add_custom_endpoint` / `remove_custom_endpoint` / `update_endpoint_last_used` |

---

## 三、可实现（附带功能，择机实现）

### MCP 管理

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| OPT-001 | 添加/更新 MCP | 服务器环境管理 MCP 服务器配置 | `upsert_mcp_server` / `upsert_mcp_server_in_config` |
| OPT-002 | 删除 MCP | 删除不需要的 MCP 服务器 | `delete_mcp_server` / `delete_mcp_server_in_config` |
| OPT-003 | 启用/禁用 MCP | 切换 MCP 服务器在不同应用中的启用状态 | `set_mcp_enabled` / `toggle_mcp_app` |
| OPT-004 | 从应用导入 MCP | 从 Claude/Codex/Gemini 等现有配置导入 | `import_mcp_from_apps` |
| OPT-005 | 测试 MCP 连接 | 验证 MCP 连通性 | `test_mcp_connection` |
| OPT-006 | Claude MCP 专属管理 | Claude 专属 MCP 配置管理 | `get_claude_mcp_status` / `read_claude_mcp_config` / `upsert_claude_mcp_server` / `delete_claude_mcp_server` / `validate_mcp_command` |

### Prompt 管理

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| OPT-007 | 添加/更新 Prompt | 添加和修改提示词 | `upsert_prompt` |
| OPT-008 | 删除 Prompt | 删除提示词 | `delete_prompt` |
| OPT-009 | 启用/禁用 Prompt | 切换提示词启用状态 | `enable_prompt` |
| OPT-010 | 从文件导入 Prompt | 从文件导入提示词 | `import_prompt_from_file` |
| OPT-011 | 查看当前 Prompt 内容 | 查看当前生效的提示词文件内容 | `get_current_prompt_file_content` |

### Skills 管理

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| OPT-012 | 列出已安装 Skills | 查看 Skills 安装情况 | `get_installed_skills` |
| OPT-013 | 安装/卸载 Skill | 管理 Skills | `install_skill_unified` / `uninstall_skill_unified` |
| OPT-014 | 启用/禁用 Skill | 切换 Skill 启用状态 | `toggle_skill_app` |
| OPT-015 | 扫描未管理 Skills | 发现散落的 Skills | `scan_unmanaged_skills` |
| OPT-016 | 从应用导入 Skills | 从现有配置导入 | `import_skills_from_apps` |
| OPT-017 | 发现可用 Skills | 从仓库浏览 | `discover_available_skills` |
| OPT-018 | 检查/更新 Skill | 检查和执行更新 | `check_skill_updates` / `update_skill` |
| OPT-019 | Skill 仓库管理 | 管理仓库源 | `get_skill_repos` / `add_skill_repo` / `remove_skill_repo` |
| OPT-020 | 从 ZIP 安装 Skills | 批量安装 | `install_skills_from_zip` |
| OPT-021 | Skill 备份恢复 | 管理和恢复备份 | `get_skill_backups` / `delete_skill_backup` / `restore_skill_backup` |

### 环境变量管理

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| OPT-022 | 环境变量冲突检查 | 检测环境变量冲突 | `check_env_conflicts` |
| OPT-023 | 删除环境变量 | 清理冲突的环境变量 | `delete_env_vars` |
| OPT-024 | 恢复环境变量备份 | 误删后恢复 | `restore_env_backup` |

### 会话管理

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| OPT-025 | 列出会话 | 查看历史会话 | `list_sessions` |
| OPT-026 | 查看会话消息 | 查看会话内容 | `get_session_messages` |
| OPT-027 | 删除会话 | 清理历史会话 | `delete_session` / `delete_sessions` |
| OPT-028 | 工具版本检测 | 检测 CLI 工具安装状态 | `get_tool_versions` / `probe_tool_installations` / `run_tool_lifecycle_action` |

### 流式健康检查

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| OPT-029 | 单供应商流式检查 | 发送流式请求验证供应商健康 | `stream_check_provider` |
| OPT-030 | 全部供应商流式检查 | 批量检查所有供应商 | `stream_check_all_providers` |
| OPT-031 | 流式检查配置 | 配置流式检查参数 | `get_stream_check_config` / `save_stream_check_config` |

### 详细用量统计

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| OPT-032 | 用量趋势 | 查看用量趋势变化 | `get_usage_trends` |
| OPT-033 | 供应商统计 | 按供应商查看用量 | `get_provider_stats` |
| OPT-034 | 模型统计 | 按模型查看用量 | `get_model_stats` |
| OPT-035 | 模型定价管理 | 自定义模型定价 | `get_model_pricing` / `update_model_pricing` / `delete_model_pricing` |
| OPT-036 | 用量脚本配置 | 自定义用量查询脚本 | `queryProviderUsage` / `testUsageScript` |
| OPT-037 | 用量数据源 | 查看用量数据来源 | `get_usage_data_sources` |
| OPT-038 | 会话用量同步 | 手动触发会话用量同步 | `sync_session_usage` |

### 通用供应商与其他扩展

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| OPT-039 | 通用供应商管理 | 跨应用通用供应商管理 | `get_universal_providers` / `upsert_universal_provider` / `delete_universal_provider` / `sync_universal_provider` |
| OPT-040 | 配置目录覆盖 | 自定义配置目录路径 | `get_app_config_dir_override` / `set_app_config_dir_override` |
| OPT-041 | 供应商复制 | 快速创建相似配置 | GUI 内部实现 |
| OPT-042 | 从 Live 移除供应商 | 仅从 live 配置移除保留数据库记录 | `remove_provider_from_live_config` |
| OPT-043 | Claude 插件配置 | 管理 Claude 插件 | `get_claude_plugin_status` / `read_claude_plugin_config` / `apply_claude_plugin_config` / `is_claude_plugin_applied` |
| OPT-044 | Claude Onboarding 跳过 | 管理引导跳过状态 | `apply_claude_onboarding_skip` / `clear_claude_onboarding_skip` |
| OPT-045 | Claude Desktop 路由 | 管理 Claude Desktop 路由 | `get_claude_desktop_default_routes` / `import_claude_desktop_providers_from_claude` / `ensure_claude_desktop_official_provider` / `get_claude_desktop_status` |
| OPT-046 | Codex 历史迁移 | 迁移 Codex 历史记录 | `has_codex_unify_history_backup` / `restore_codex_unified_history` |

### OpenClaw 专属

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| OPT-047 | 配置健康扫描 | 检查 OpenClaw 配置健康 | `scan_openclaw_config_health` |
| OPT-048 | 默认模型管理 | 设置和查看默认模型 | `get_openclaw_default_model` / `set_openclaw_default_model` |
| OPT-049 | 模型目录管理 | 管理模型目录 | `get_openclaw_model_catalog` / `set_openclaw_model_catalog` |
| OPT-050 | Agents 默认值 | 配置 Agents 默认值 | `get_openclaw_agents_defaults` / `set_openclaw_agents_defaults` |
| OPT-051 | 环境变量管理 | 管理 OpenClaw 环境变量 | `get_openclaw_env` / `set_openclaw_env` |
| OPT-052 | 工具配置管理 | 管理 OpenClaw 工具配置 | `get_openclaw_tools` / `set_openclaw_tools` |
| OPT-053 | 工作区文件管理 | 读写 OpenClaw 工作区文件 | `read_workspace_file` / `write_workspace_file` / `open_workspace_directory` |
| OPT-054 | 每日记忆文件管理 | 管理每日记忆文件 | `list_daily_memory_files` / `read_daily_memory_file` / `write_daily_memory_file` / `delete_daily_memory_file` / `search_daily_memory_files` |

### Hermes 专属

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| OPT-055 | Live 供应商导入 | 从 live 配置导入 Hermes 供应商 | `import_hermes_providers_from_live` / `get_hermes_live_provider_ids` / `get_hermes_live_provider` |
| OPT-056 | 模型配置查看 | 查看 Hermes 模型配置 | `get_hermes_model_config` |
| OPT-057 | 记忆管理 | 管理 Hermes 记忆 | `get_hermes_memory` / `set_hermes_memory` / `get_hermes_memory_limits` / `set_hermes_memory_enabled` |

### OMO 配置

| 编号 | 功能 | 说明 | 对应 GUI 命令 |
|------|------|------|---------------|
| OPT-058 | OMO 配置读取 | 读取 OMO 本地配置 | `read_omo_local_file` / `get_current_omo_provider_id` |
| OPT-059 | OMO 停用 | 停用 OMO | `disable_current_omo` |
| OPT-060 | OMO Slim 管理 | 管理 OMO Slim | `read_omo_slim_local_file` / `get_current_omo_slim_provider_id` / `disable_current_omo_slim` |

---

## 四、没必要实现

### 非核心功能

| 编号 | 功能 | 理由 |
|------|------|------|
| N/A-001 | WebDAV 云同步 (`webdav_test_connection` / `webdav_sync_upload` / `webdav_sync_download` / `webdav_sync_save_settings` / `webdav_sync_fetch_remote_info`) | 非代理核心功能，服务器环境可用 rsync/git/scp 等系统工具替代 |
| N/A-002 | S3 云同步 (`s3_test_connection` / `s3_sync_upload` / `s3_sync_download` / `s3_sync_save_settings` / `s3_sync_fetch_remote_info`) | 同上，非核心，可用 aws-cli 等工具替代 |
| N/A-003 | Copilot OAuth 认证 (`copilot_start_device_flow` 等全套) | AUTH 管理非代理核心，CLI 用户可直接在配置中填入 Token |
| N/A-004 | Codex OAuth 认证 (`get_codex_oauth_quota` / `get_codex_oauth_models`) | 同上 |
| N/A-005 | 通用 OAuth 认证 (`auth_start_login` 等全套) | 同上 |
| N/A-006 | 订阅配额查询 (`get_subscription_quota` / `get_coding_plan_quota` / `get_balance`) | 非代理核心，依赖 OAuth 认证 |
| N/A-007 | Keychain API Key 存储 (`set_api_key` / `get_api_key` / `delete_api_key`) | 非核心，且 Linux 下已被 cfg 掉，CLI 用户在配置文件中管理 Key 即可 |

### GUI 专属（强依赖桌面运行时）

| 编号 | 功能 | 理由 |
|------|------|------|
| N/A-008 | 窗口主题控制 (`set_window_theme`) | 无窗口 |
| N/A-009 | 轻量模式 (`enter_lightweight_mode` / `exit_lightweight_mode` / `is_lightweight_mode`) | 仍在 Tauri 进程内，非真正无头 |
| N/A-010 | Deep Link 导入 (`parse_deeplink` / `merge_deeplink_config` / `import_from_deeplink` / `import_from_deeplink_unified`) | 需浏览器和系统 URL 协议交互 |
| N/A-011 | 自动启动 (`set_auto_launch` / `get_auto_launch_status`) | 桌面专属，CLI 用 systemd/cron |
| N/A-012 | 应用更新 (`check_app_update_available` / `check_for_updates` / `install_update_and_restart`) | CLI 用包管理器更新 |
| N/A-013 | 托盘菜单 (`update_tray_menu`) | 无系统托盘 |
| N/A-014 | 文件对话框 (`save_file_dialog` / `open_file_dialog` / `open_zip_file_dialog`) | 需图形界面，CLI 直接传路径 |
| N/A-015 | 打开外部链接/文件夹 (`open_external` / `open_config_folder` / `open_app_config_folder`) | 桌面专属，CLI 用 shell 命令 |
| N/A-016 | 剪贴板 (`copy_text_to_clipboard`) | 桌面专属，CLI 用管道重定向 |
| N/A-017 | Hermes Web UI (`open_hermes_web_ui` / `launch_hermes_dashboard`) | 需浏览器 |
| N/A-018 | 打开供应商终端 (`open_provider_terminal`) | 需终端模拟器，CLI 用户已在终端 |
| N/A-019 | 启动会话终端 (`launch_session_terminal`) | 同上 |
| N/A-020 | 通知/语言/主题/窗口设置/键盘快捷键 | 全部为 GUI 外观和交互偏好 |

---

## 五、汇总统计

| 分类 | 编号范围 | 数量 | 说明 |
|------|----------|------|------|
| 必须实现 | REQ-001 ~ REQ-019 | 19 | 代理与协议转换核心，CLI 必须与 GUI 一致 |
| 可实现 | OPT-001 ~ OPT-060 | 60 | 附带功能（MCP/Prompt/Skills/环境变量/会话等），择机实现 |
| 没必要实现 | N/A-001 ~ N/A-020 | 20 | 非核心（云同步/AUTH/Keychain）+ GUI 专属 |

---

## 六、实现优先级建议

### 第一阶段：代理核心能力补齐（REQ-001 ~ REQ-009）

供应商排序、Live 导入/读取/同步、模型获取、代理配置读写、成本倍率/计费来源、Live 接管检测。这是代理运行和协议转换的基础能力，应最先实现。

### 第二阶段：代理运维与监控（REQ-010 ~ REQ-019）

熔断器统计、供应商健康、可用故障转移列表、通用配置片段、按应用用量、请求日志、限额检查、备份删改、自定义端点管理。这是代理运维监控的核心能力。

### 第三阶段：附带功能（选择性实现 OPT 系列）

根据用户需求从 OPT 系列中选择实现。MCP/Prompt/Skills 管理、环境变量管理、会话管理、流式检查、详细用量统计等。

---

## 七、结论

基于"代理和协议转换是 GUI 主体功能"的原则，CLI 必须实现的为 19 项核心功能（REQ 系列），围绕供应商配置、代理运行、故障转移、请求处理、用量监控展开。60 项附带功能（OPT 系列）包含 MCP/Prompt/Skills/环境变量/会话等，可择机实现。20 项功能（N/A 系列）包含云同步、AUTH 认证、Keychain 以及 GUI 专属功能，不建议在 CLI 中实现——服务器环境可用 rsync/git 替代云同步，在配置文件中直接填 Token 替代 OAuth，用 systemd/cron 替代自动启动。
