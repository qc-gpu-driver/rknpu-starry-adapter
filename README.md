# rknpu-starry-adapter

`rknpu-starry-adapter` 是 `StarryOS <-> rknpu` 的桥接层。

这个 crate 不实现 NPU 算法本身，职责是把：

- `rknpu` 的核心驱动能力（MMIO/IRQ/submit/GEM）
- StarryOS 的设备模型、devfs、用户态拷贝、调度与电源域

接到一起，并对外提供可注册的设备与 probe 入口。

## 主要职责

- FDT probe：识别 `rockchip,rk3588-rknpu`，映射寄存器，注册 IRQ。
- 上电流程：拉起 NPU/NPUTOP/NPU1/NPU2 power domain。
- IRQ 桥接：平台 IRQ 回调转发到 `RknpuIrqHandler::handle()`。
- 设备节点实现：
  - `/dev/dri/card0`（基础 DRM 信息 ioctl）
  - `/dev/dri/card1`（DRM + RKNPU driver ioctl）
  - `/dev/rknpu`（复用 card1 设备语义）
- 在 `card1` 内接入 `RknpuService<StarryPlatform>`，把等待/唤醒/worker 运行时接到 StarryOS。

## 代码结构

```text
rknpu-starry-adapter/
├── src/
│   ├── lib.rs           // 对外导出与模块拼装
│   ├── card0.rs         // /dev/dri/card0
│   ├── card1.rs         // /dev/dri/card1 + /dev/rknpu 主要 ioctl/mmap
│   ├── devfs.rs         // 设备节点注册函数 register_rknpu_devices
│   └── drm.rs           // DRM ioctl 辅助结构与解码
├── npuprobe.rs          // FDT probe + MMIO + IRQ + driver register
├── irq.rs               // 全局 IRQ slot 与 trampoline
├── power.rs             // power domain 上电 + irq_yield
└── tool.rs              // iomap 工具封装
```

## 对外入口

- `rknpu_probe`：由 `module_driver!` 注册为 FDT probe 回调。
- `enable_pm`：可被外部显式调用的上电 helper。
- `register_rknpu_devices(fs, root)`：将 `/dev/rknpu` 和 `/dev/dri/card{0,1}` 挂到 devfs。

注意：`npuprobe.rs` 只负责平台注册（`plat_dev.register(npu)`）和 IRQ 接线，不会自动调用 `register_rknpu_devices()`；设备节点需要在 OS 的 devfs 初始化路径显式注册。

## StarryOS 侧语义依赖

`card1` 的平台适配实现依赖以下语义：

- 用户态内存拷贝：`axhal::asm::user_copy`。
- 提交等待：`starry_core::futex::WaitQueue`。
- worker 事件：`event_listener::Event`。
- worker 线程与让出：`axtask::spawn` / `axtask::yield_now`（`aarch64`）。
- 设备获取：`rdrive::get_one::<rknpu::Rknpu>()`。

如果迁移到其他 OS，需要按 `rknpu::service` 的平台 trait 重新实现这些语义。


## `extern crate` 约定

本 crate 保持 `no_std` + `extern crate` 显式引入风格，入口见 `src/lib.rs`。  
如果后续开启更激进的 LTO/裁剪并出现“间接依赖符号被裁掉”的问题，再补 `#[used]` 锚点宏（例如 keep-extern-symbol 模式）即可；当前代码中未强制启用该宏。

## 构建检查

```bash
cargo check --manifest-path /home/inkbottle/othersrc/npu/drivers/rknpu-starry-adapter/Cargo.toml
```

如果出现路径依赖错误，先校准上面的目录布局，再做编译。
