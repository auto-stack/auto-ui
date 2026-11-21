# examples/tag-input — TagInput 普通设置消费者（PLAN-097 T-04）

普通设置消费者，覆盖合同主路径：接受流、去重、删除、上限、业务拒绝、候选/键盘
与配置错误。三个实例 + 重置/只读/禁用/外部错误控制。

## 实例

| 实例 | 配置 | 覆盖 |
|---|---|---|
| A | maxTags=5、suggestions=[Design,Development,Work]、初值 [Reading, Work] | 候选/Enter 接受流、去重、删除、只读/禁用、外部错误 |
| B | maxTags=2、初值 [Reading, Work] | 上限阻止新增仍可删除、业务拒绝 Design |
| C | 初值 [Reading] | 候选重开/点击添加、重复 tags 配置错误与恢复 |

## 运行与验证

```bash
cd examples/tag-input
auto gen && auto run -r vm            # native VM（MCP: AUTOUI_MCP_PORT）
auto gen && auto run                  # Vue 开发服务器
```

产品门：`tests/product/verify_tag_input.py`（native/vue 两轨，`--case demo`）与
`tests/product/tag_input_preview.py`（Vue 预览构建）。
