var ServiceInfo = [
    { id: 0x10, name: "DiagnosticSessionControl",  desc: "诊断会话控制" }
    { id: 0x11, name: "EcuReset",  desc: "电控单元复位" }
    { id: 0x14, name: "ClearDiagnosticInformation",  desc: "清除诊断信息" }
    { id: 0x19, name: "ReadDTCInformation",  desc: "读取DTC信息" }
    { id: 0x22, name: "ReadDataByIdentifier",  desc: "读取数据" }
    { id: 0x23, name: "ReadMemoryByAddress",  desc: "读取内存" }
    { id: 0x27, name: "SecurityAccess",  desc: "安全访问" }
    { id: 0x28, name: "CommunicationControl",  desc: "通信控制 " }
    { id: 0x2A, name: "ReadDataByPeriodicIdentifier",  desc: "读取数据（周期标识符）" }
    { id: 0x2C, name: "DynamicallyDefineDataIdentifier",  desc: "动态定义数据标识符" }
    { id: 0x2E, name: "WriteDataByIdentifier",  desc: "写入数据" }
    { id: 0x2F, name: "InputOutputControlByIdentifier",  desc: "输入输出控制" }
    { id: 0x31, name: "RoutineControl",  desc: "例程控制" }
    { id: 0x3D, name: "WriteMemoryByAddress",  desc: "写入内存" }
    { id: 0x3E, name: "TesterPresent",  desc: "诊断设备在线" }
    { id: 0x85, name: "ControlDTCSetting",  desc: "控制DTC设置" }
]

var Config = [
    { name: "服务ID", width: 100.0, showas: "Hex" }
    { name: "服务名称", width: 250.0, showas: "Text" }
    { name: "服务描述", width: "Strech", showas: "Text" }
]

widget ServiceTable {
    model {
        var config = Config
        var data = ServiceInfo
    }

    view {
        table(config, data)
    }
}

print(ServiceTable)