# 8-bit 音乐板（不是，其实是1-bit）

一个基于 `STM32F030K6T6` 微控制器的音乐播放器项目，支持播放多首经典音乐，并配有 RGB LED 灯光效果和按键控制。

## 项目特性

- 🎵 支持多首内置音乐播放（小星星、生日快乐、玛丽有只小羊羔、欢乐颂、致爱丽丝）
- 🎮 4个按键控制：上一首、下一首、暂停/继续、RGB灯开关
- 💡 RGB LED 灯光效果
- 🔊 PWM 音频输出
- ⚡ 按键软件防抖设计
- 📱 支持实时按键检测和音乐切换

## 硬件要求

- `STM32F030K6T6` 微控制器
- 外部 8MHz 晶振
- 4个按键（上拉电阻）
- RGB LED（共阴极）
- 蜂鸣器或扬声器
- 电源供应

## 引脚配置表

| 引脚                  | 功能          | 描述                     |
| --------------------- | ------------- | ------------------------ |
| **GPIO 输入（按键）** |               |                          |
| PB6                   | 上一首按键    | 下拉输入，按下时为高电平 |
| PB7                   | 下一首按键    | 下拉输入，按下时为高电平 |
| PA11                  | 暂停/继续按键 | 下拉输入，按下时为高电平 |
| PA12                  | RGB灯开关按键 | 下拉输入，按下时为高电平 |
| **PWM 输出**          |               |                          |
| PB4                   | TIM3_CH1      | PWM输出 - RGB LED 红色   |
| PB5                   | TIM3_CH2      | PWM输出 - RGB LED 绿色   |
| PB0                   | TIM3_CH3      | PWM输出 - RGB LED 蓝色   |
| PB1                   | TIM3_CH4      | PWM输出 - 蜂鸣器         |
| **电源与时钟**        |               |                          |
| VDD                   | 电源正极      | 3.3V 电源输入            |
| VSS                   | 电源负极      | 接地                     |
| OSC_IN                | 晶振输入      | 8MHz 外部晶振            |
| OSC_OUT               | 晶振输出      | 8MHz 外部晶振            |
| **调试接口**          |               |                          |
| SWDIO                 | SWD 数据      | 调试和编程               |
| SWCLK                 | SWD 时钟      | 调试和编程               |

## 软件架构

### 主要模块

1. **主控制模块** (`main.rs`)
   - 系统初始化
   - GPIO 配置
   - PWM 配置
   - 中断配置
   - 主音乐播放循环

2. **音乐模块** (`music.rs`, `music/`)
   - 音符定义 (`note.rs`)
   - 音乐列表 (`list.rs`)
   - 音乐播放控制
   - 按键事件处理

3. **RGB 模块** (`rgb.rs`)
   - RGB LED 控制
   - 亮度和颜色管理

### 定时器配置

- **TIM3**: PWM 输出
  - CH4: 音频输出
  - CH1-CH3: RGB LED 控制
- **TIM16**: 按键防抖定时器，频率 50Hz （20ms延迟防抖）

## 编译和烧录

### 环境要求

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装目标架构
rustup target add thumbv6m-none-eabi

# 安装 probe-rs（用于烧录和调试）
cargo install probe-rs-tools
```

### 编译项目

```bash
# 编译 debug 版本
cargo build

# 编译 release 版本
cargo build --release
```

### 烧录到设备

```bash
# 使用 probe-rs 烧录
cargo run --release

# 或者使用 VS Code launch.json 配置的调试器
```

## 使用说明

1. **开机**: 设备上电后自动开始播放第一首音乐
2. **切换音乐**:
   - 按下 PB7（下一首）切换到下一首音乐
   - 按下 PB6（上一首）切换到上一首音乐
3. **暂停/继续**: 按下 PA11 暂停当前音乐或继续播放
4. **RGB 灯光**: 按下 PA12 开关 RGB LED 灯

### 内置音乐列表

1. 小星星 (Twinkle Twinkle Little Star)
2. 生日快乐 (Happy Birthday)
3. 玛丽有只小羊羔 (Mary Had a Little Lamb)
4. 欢乐颂 (Ode to Joy)
5. 致爱丽丝 (Für Elise)
6. ……

## 技术细节

### 内存配置

- **Flash**: 32KB (程序存储)
- **RAM**: 4KB (运行时内存)
- **系统频率**: 48MHz
- **外部晶振**: 8MHz

### 音频生成

音频通过 PWM 波形生成，每个音符对应特定的频率：

- 使用定时器 TIM3 的通道1输出 PWM 信号
- PWM 频率根据音符频率动态调整
- 支持休止符 (RST) 播放

### 按键防抖

采用定时器中断方式实现软件防抖：

- TIM16 以 50Hz 频率扫描按键状态
- 检测按键状态变化（上升沿触发）
- 避免按键抖动造成的误触发

## 开发和调试

### 日志输出

使用 `defmt` 框架进行日志输出：

- 通过 RTT (Real-Time Transfer) 输出调试信息
- 支持格式化日志和性能分析

### 添加新音乐

1. 在 `src/music/note.rs` 中定义音符频率
2. 在 `src/music/list.rs` 中添加新的音乐数据
3. 音乐格式：`&[(frequency, duration)]`
   - `frequency`: 音符频率（Hz）
   - `duration`: 音符时长（毫秒）

## 许可证

本项目采用 [MulanPSL-2.0](https://license.coscl.org.cn/MulanPSL2) 许可证。
