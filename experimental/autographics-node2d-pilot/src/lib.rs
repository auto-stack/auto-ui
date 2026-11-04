//! autographics-node2d-pilot — PLAN-016/017 研究 adapter 库。
//!
//! 分段（016 合同 §2，017 扩展）：load（领域解析+模块表）→ validate
//! （执行前定位诊断）→ lower（实例表→单文件 Auto 用户源 + 场景文档）。
//! 游戏规则全部落在生成的真实 Auto 源里；Rust 只做调度、类型化桥与
//! 图形服务连接。本 crate 刻意不依赖 AG driver（tip vs pinned-SDK
//! 两棵依赖树绝不同图）；生成的源/场景由 driver 侧测试作为冻结数据消费。
//!
//! PLAN-017：`Profile::G112D`（`--profile g11-2d` 显式选择）为完整
//! G11/2d 作者入口（声明驱动模板/角色/事件/调度）；`Profile::Legacy`
//! 保持 016 最小切片逐字节兼容（verify_plan016 再生控制），不被自动升级。
pub mod load;
pub mod lower;
pub mod lower_queries3d;
pub mod lower_basic2d;
pub mod lower_components;
pub mod lower_components3d;
pub mod lower_components3d_bindings;
pub mod lower_components3d_events;
pub mod lower_components3d_audio_hud;
pub mod node3d_binding_refs;
pub mod lower_components3d_runtime;
pub mod lower_components_game;
pub mod lower_g11;
pub mod lower_components2d_runtime;
pub mod lower_node3d_presentation;
pub mod lower_presentation;
pub mod scene;
pub mod source_map;
pub mod translate;

/// 降层 profile（PLAN-017 T-02 冻结；PLAN-020 增补 components-2d；
/// PLAN-021 增补 queries-2d；PLAN-022 增补 sprites-2d；PLAN-024 增补
/// components-3d）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// PLAN-016 最小切片：字节兼容，服务映射不变。
    Legacy,
    /// PLAN-017 完整 G11/2d 作者入口。
    G112D,
    /// PLAN-019 基础二维：scene-2d/character-2d/camera-2d。
    Basic2D(lower_basic2d::BasicProfile),
    /// PLAN-020 有状态组件：components-2d（T-02 冻结的两套既有机制）。
    Components2D,
    /// PLAN-021 typed 二维查询：queries-2d（components-2d 超集 + query2d
    /// 服务/具名行/typed 结果；api/运行时装配不变）。
    Queries2D,
    /// PLAN-022 typed 精灵/帧动画：sprites-2d（组件局部 image/frame/flip
    /// 绑定 + render2d 渲染实例服务；sprites2d/1 装配）。
    Sprites2D,
    /// PLAN-023 components-2d-game：game2d 作者入口（T-02 冻结合同
    /// F-1..F-6 的生成器实现；frozen components-2d 字节不变）。
    Components2DGame,
    /// PLAN-024 三维组件：components-3d（view-3d/有限图元/静态平色 GLB/
    /// 同维层级；behavior/1 装配；T-02 冻结的唯一接法）。
    Components3D,
    /// PLAN-025 通用声音/HUD 接线：presentation-2d（presentation2d 作者块；
    /// 无业务枚举；无角色 = autographics.presentation2d/1，有角色 =
    /// autographics.hud2d/1 受检组合）。
    Presentation2D,
    /// PLAN-032 完整二维游戏组合：components-2d-runtime（components2d 作者
    /// 块；autographics.components2d/1 受检组合；T-02 冻结骨架）。
    Components2DRuntime,
    /// PLAN-027 三维角色/运行相机/内置视觉声明接线：components-3d-runtime
    /// （character3d/solid/camera3d/player3d/mob3d 声明；六动作输入表；
    /// refs 角色服务；沿 024 共享管线 + 注入位）。
    Components3DRuntime,
    /// PLAN-028 三维实例/触发事件/新局接线：components-3d-events（object3d
    /// 模板/objects3d 槽池/triggers pair 池/spawn/bind/event.*/object.*
    /// 调用端口；triggers3d/1 装配；沿 024 共享管线 + 注入位）。
    Components3DEvents,
    /// PLAN-030 三维具名引用与跨组件接线：components-3d-bindings（028 全部
    /// 面 + Slot/Template/Pair/Actor/msg 只读名义参数、纯逻辑 Node、根显式
    /// 调度；triggers3d/1 装配；028 档字节不变）。
    Components3DBindings,
    /// PLAN-034 三维纹理资源/具名动画与组件呈现接线：components-3d-presentation
    /// （textured3d.glb/animated3d.glb 具名资源、ClipRef 具名 clip、refs 姿态
    /// 服务、animated3d 模型叶角色 body；bindings 载体 + 既有公开导入产出的
    /// 受检闭包元数据；动画档 node_animation3d/1 装配）。
    Components3DPresentation,
    /// PLAN-035 三维声音/HUD/姿态组合：components-3d-audio-hud（presentation3d
    /// 作者块 audio/cue/hud + 024..034 全部三维作者构造；有界事件快照/批
    /// 销毁作者面；角色姿态随 bundle 收口；actor-free kind=-1；components3d/1
    /// 装配，T-03 冻结 117/28）。
    Components3DAudioHud,
    /// PLAN-043 模型对象模板组合入口：model-objects-3d（events 对象/触发
    /// /模板域 + 034 受检资源元数据；平色 sha 键与 opaque closure 键模板；
    /// animated 整域拒绝；T-02 冻结的显式 B 组合身份）。
    ModelObjects3D,
    /// PLAN-039 有限三维查询与同根双出口：queries-3d（query3d.segment/
    /// sweep typed 面、具名叶/组引用、root view-3d＋view-2d 诊断标记；
    /// segment-only → behavior/3、含 sweep → behavior/4 装配；沿 024
    /// 共享管线 + 档注入）。
    Queries3D,
    /// PLAN-040 编辑工程：与 queries-3d 完全相同的静态双出口机制与纯声明
    /// behavior:null 路径，仅换冻结身份串（document_id 前缀/生成说明/再生
    /// 控制注记/消费者登记）。G04 忠实夹具的消费入口；不为纯场景新增语法。
    EditorProjects,
}

pub use load::{load_project, load_project_dual, Project, ProjectError};
pub use lower_queries3d::{
    lower_project_editor_projects, lower_project_queries3d, BEHAVIOR_API_V3, BEHAVIOR_API_V4,
};
pub use lower::{lower_project, LowerOutput};
pub use lower_basic2d::{lower_project_basic, BasicProfile, LowerOutputBasic2D};
pub use lower_components::{
    lower_project_components, lower_project_queries, lower_project_sprites,
};
pub use lower_components3d::lower_project_components3d;
pub use lower_components3d_bindings::{
    lower_project_components3d_bindings, lower_project_components3d_presentation_entry,
};
pub use lower_node3d_presentation::{
    load_resources_meta, lower_project_components3d_presentation, resources_request_json,
    PresentationMeta3,
};
pub use lower_components3d_audio_hud::lower_project_components3d_audio_hud;
pub use lower_components3d_events::{
    lower_project_components3d_events, lower_project_components3d_model_objects,
};
pub use lower_components3d_runtime::lower_project_components3d_runtime;
pub use lower_components_game::{lower_project_components_game, LowerOutputGame};
pub use lower_components2d_runtime::{
    lower_project_components2d_runtime, LowerOutputComponents2dRuntime,
};
pub use lower_presentation::{lower_project_presentation, LowerOutputPresentation};
pub use lower_g11::{lower_project_g112d, LowerOutputG11};
