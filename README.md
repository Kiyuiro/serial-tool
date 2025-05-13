# Serial Tool [![Apache](https://img.shields.io/static/v1.svg?label=license&message=Apache+2&color=blue)](https://github.com/mt6595/AtopSerial/blob/main/LICENSE)

一个基于 **Rust + Tauri + Vue3** 的跨平台串口调试工具，界面简洁直观，功能实用，适合开发人员在嵌入式开发、设备调试等场景下使用。

## ✨ 特性

- 支持串口基本参数配置（波特率、数据位、停止位、校验位等）
- 实时发送/接收数据，支持 ASCII 和十六进制格式
- 自动换行显示接收内容
- 支持接收区清空
- 支持颜色高亮显示发送和接收内容
- 跨平台支持：Windows / Linux / macOS
- 轻量、无需依赖运行时环境

## 🧱 技术栈

- [Rust](https://www.rust-lang.org/) - 高性能后端逻辑处理
- [Tauri](https://tauri.app/) - 构建跨平台原生桌面应用
- [Vue3](https://vuejs.org/) + [Element Plus](https://element-plus.org/) - 构建现代 Web UI 界面

## 📦 安装与运行

### 1. 克隆项目

```bash
git clone https://github.com/Kiyuiro/serial-tool.git
cd serial-tool
```

### 2. 安装依赖

```bash
npm install
```

### 3. 运行开发模式

```bash
npm run tauri dev
```

### 4. 打包发布应用

```bash
npm run tauri build
```

## 📷 截图预览

![串口助手界面](./public/screenshot.png)

## 📄 开源许可证

本项目基于 [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0) 开源发布。

```
Copyright [2025] [Kiyuiro]

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0
```

