# PLAN-005 r3 执行合同（T-02整链可行性已证明，完整门仍待T-03..06）

基线：`2b88bd69b2a48172bf761615cc4e78a1df563b62`；实施树 `D:/autostack/.wt/ui-005/auto-ui`，`codex/plan-005-dev`。004交付与SDK/锁及必需原件见 [fixed-source.md](fixed-source.md)。本文件冻结待实现的数据流与关闭条件，不把设计文本当作T-02通过证据。

## 支持表

- glTF2 GLB恰有JSON+BIN、单内嵌geometry buffer、单场景、静态三角形；沿用旧全声明accessor/稀疏/节点/几何先检。
- POSITION/NORMAL、UV0 FLOAT或normalized U8/U16；原件未使用TANGENT须FLOAT VEC4/同count/非normalized/跨度/有限值/w=±1先检，旧profile仍拒绝切线；本期不提供法线贴图。opaque单面/双面、baseColorFactor乘sRGB baseColorTexture。双面以front_facing校正法线，颜色/阴影都按材质决定剔除。旧平色profile仍拒绝双面/纹理。
- 仅PNG，完整复用004 PNG先检域；GLB bufferView或受控本地相对URI，检查全部声明包括未引用图。无data/network/绝对路径、百分号/query/fragment、父目录逃逸、反斜杠/冒号与链接/reparse。依赖读取只能由driver受检路径执行，通用GLB import不接受URI。
- KHR_texture_transform只在baseColorTexture信息上，有限offset/rotation/scale与texCoord0；其它扩展/材质通道拒绝，动画/skin/morph继续拒绝。
- nearest/linear与clamp/repeat/mirrored-repeat；9984/9986归一为基础nearest，9985/9987归一为基础linear，单层纹理固定LOD0并在交付文档明确，不承诺mipmap过滤。PNG每像素alpha必须255。

## 实际路径与实现接缝

现有 `assets::import→decode_in→preflight→agp::load_glb_bytes`；`Edit::AssetObject→Registry→Document→Snapshot→prepare_closure→entry commit→load_declared`；`host/viewport::prepare_frame→MeshStore staged→Prepared→publish`；`MeshPipeline`颜色与shadow pass。

拟新增受检纹理profile：先无IO解析原始GLB并验证扩展位置/材料/图像边，再构造有界geometry-only decoder scratch（移除已验证纹理声明，不改原件），复用旧preflight的全声明geometry oracle；原UV/节点/primitive/transform关联单独保留。只有完整准入后才能解码PNG、mesh及准备GPU。绝不把scratch当原件或存储身份，旧入口不放宽。

`AssetRef`保留根原始SHA/长度，新可选closure字段绑定有序image边/子SHA/维度和closure哈希；Registry和对象资源键按closure标识，因此同根不同外图合法并存。扩展schema6，schema1..5拒绝closure字段；无closure的旧语义保持。根原始字节、所有外图bytes作为不可变owner随Asset保留，SaveAs不重读导入目录。项目entry持有依赖映射，存储仅root及PNG的SHA blob；bufferView图raw已包含根，不重复存盘。读取即重新校验边/闭包与所有blob，cached路径也不跳过磁盘依赖核对。

AGP拟在MeshData追加可选visual信息（UV、材质采样、图像Arc、原始node/mesh/primitive索引和可逆world变换）；保留原GLB，因此006可恢复local geometry/TRS，不仅留下不可逆烘焙。GPU mesh以closure/expanded primitive缓存；纹理按原图SHA共享，sampler按有限配置共享，同device代次校验。外部pin按最后owner释放。

## 预算与事务

根GLB和独立PNG按原始SHA去重union共8/32MiB；单raw8MiB，JSON1MiB；原几何/节点/instance帽保持。CPU/GPU64MiB包括UV/节点/材质/像素与实际上传对齐；先预测再分配，实际复核；不以每个closure另获额度。依赖在history/pending/current及cache同样计费，嵌入图raw不重复计，decoded/GPU仍付费。未引用声明也完成准入和预算。

GPU资源只能插入staged候选；任何校验/解码/prepare/OOM失败保last-good整帧。warm相机/运动不重解码或重传静态资源。Stop释放Run附加owner而Editor保留合法owner，关闭后依外部pin最后释放。

## T-02关闭条件及后续门

T-02已在提交9a0f722上通过固定platform/coin/flag原件的公开import/edit/save→搬迁open→无窗口GPU输出，保留root/closure/node关联；每件另以同根不同PNG证明像素敏感。`textured_model_resources`两测试通过，原始日志与SHA见Plan §9；初步边与资源在原预算内，完整union/故障/生命周期门归后续T-03..06，不能以T-02代替。首次原件切线拒绝及其有界澄清也保留于Plan收据。

后续V-01坏URI/声明/跨度/类型/超限与同根不同图；V-02搬迁/缺损/错边/中断恢复；V-03非对称UV/变换/采样/背面/阴影/深度与独立CPU采样及变异；V-04 20轮owner/上传/候选故障/共享；V-05 fresh离线SDK consumer与004/旧静态兼容。每项以实际断言和冻结提交收据关闭；checker必须拒绝0selected、旧runner、partial和失真报告。
