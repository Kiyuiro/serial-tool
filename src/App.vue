<template>
  <div class="container">

    <!-- 配置 -->
    <div class="container-config">
      <el-divider content-position="left">串口配置</el-divider>
      <el-form label-width="60px">
        <el-form-item label="端口名">
          <el-select v-model="config.port" placeholder="请选择串口" style="width: 100%" @visible-change="scanPorts">
            <el-option v-for="port in availablePorts" :key="port.name" :label="`${port.name} ${port.port_type}`"
                       :value="port.name"/>
          </el-select>
        </el-form-item>
        <el-form-item label="波特率">
          <el-select v-model="config.baudRate" placeholder="波特率" style="width: 100%">
            <el-option v-for="rate in baudRates" :key="rate" :label="rate" :value="rate"/>
          </el-select>
        </el-form-item>
        <el-form-item label="数据位">
          <el-select v-model="config.dataBits" style="width: 100%">
            <el-option :label="7" :value="7"/>
            <el-option :label="8" :value="8"/>
          </el-select>
        </el-form-item>
        <el-form-item label="停止位">
          <el-select v-model="config.stopBits" style="width: 100%">
            <el-option :label="1" :value="1"/>
            <el-option :label="2" :value="2"/>
          </el-select>
        </el-form-item>
        <el-form-item label="校验位">
          <el-select v-model="config.parity" style="width: 100%">
            <el-option label="None" value="none"/>
            <el-option label="Even" value="even"/>
            <el-option label="Odd" value="odd"/>
          </el-select>
        </el-form-item>
      </el-form>
      <el-button type="primary" @click="togglePort" style="width: 100%">
        {{ isPortOpen ? '关闭串口' : '打开串口' }}
      </el-button>
      <el-divider content-position="left">发送配置</el-divider>
      <div>
        <el-checkbox v-model="config.hexSend">十六进制发送</el-checkbox>
      </div>
      <div>
        <span style="font-weight: 500; font-size: 14px; color: #606266; margin-right: 100px;">发送字符串颜色</span>
        <el-color-picker v-model="config.sendTextColor"/>
      </div>
      <el-divider content-position="left">接收配置</el-divider>
      <div>
        <el-checkbox v-model="config.hexDisplay">十六进制显示</el-checkbox>
      </div>
      <div>
        <span style="font-weight: 500; font-size: 14px; color: #606266; margin-right: 100px;">接收字符串颜色</span>
        <el-color-picker v-model="config.receivedTextColor"/>
      </div>
    </div>

    <!-- 发送和接收 -->
    <div class="container-received">
      <!-- 接收数据 -->
      <div ref="receivedContext"
           style="flex: 1; border: 1px solid #DCDFE6; border-radius: 5px; margin-bottom: 3px; padding: 10px; overflow: auto">
        <p v-for="item in receivedData" style="white-space: pre-wrap; word-break: break-all;">
          <span v-if="item.direction == 'in'" :style="{color: config.receivedTextColor}">« {{ item.data }}</span>
          <span v-else :style="{color: config.sendTextColor}">» {{ item.data }}</span>
        </p>
      </div>
      <!-- 发送数据 -->
      <div style="height: 140px">
        <div style="display: flex; flex-direction: row">
          <el-input v-model="config.sendData" type="textarea" :rows="4" class="locked-textarea" style="flex: 1"
                    placeholder="Alt + Enter 发送" @input="sendDataInputEvent"/>
          <el-button type="primary" @click="sendToPort" :disabled="!isPortOpen" class="send-data-button">发送
          </el-button>
        </div>
        <div style="margin-top: 8px; padding: 0 10px">
          <span class="left" style="margin-right: 20px">发送: {{ sendCount }}</span>
          <span class="left">接收: {{ receivedCount }}</span>
          <el-button class="right" type="danger" @click="clearReceived">清空接收区</el-button>
          <el-checkbox class="right" v-model="config.autoScroll" style="margin-right: 8px">自动滚动</el-checkbox>
          <el-checkbox class="right" v-model="config.saveSendMsg" style="margin-right: 8px">保留发送区</el-checkbox>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import {ElMessage} from 'element-plus'
import {nextTick, onMounted, ref, watch} from 'vue'
import {invoke} from "@tauri-apps/api/core";
import {listen} from '@tauri-apps/api/event'

interface PortName {
  name: string,
  port_type: string
}

interface Message {
  data: string
  direction: 'in' | 'out'
}

interface RustConfig {
  baud_rate: number
  send_data: string
  hex_send: boolean
  hex_display: boolean
  auto_scroll: boolean
  save_send_msg: boolean
  received_text_color: string
  send_text_color: string
}

// 串口配置
const config = ref({
  port: '', // 串口名称
  baudRate: 9600, // 波特率
  dataBits: 8, // 数据位
  stopBits: 1, // 停止位
  parity: 'none', // 校验位
  sendData: '', // 发送数据
  hexSend: false, // 十六进制发送
  hexDisplay: false, // 十六进制显示
  autoScroll: true, // 自动滚动
  saveSendMsg: false, // 是否保留发送区
  receivedTextColor: '#000000', // 接收字符串颜色
  sendTextColor: '#FFAA00', // 发送字符串颜色
})

const isPortOpen = ref(false) // 串口是否打开
const availablePorts = ref<PortName[]>([]) // 可用串口列表
const baudRates = [9600, 19200, 38400, 57600, 115200] // 波特率列表
const receivedData = ref<Message[]>([]) // 接收数据
const receivedCount = ref(0) // 接收数据计数
const sendCount = ref(0) // 发送数据计数
const receivedContext = ref() // 接收数据容器

onMounted(async () => {
  serialDataListener();
  serialErrorListener();
  let res = await invoke('load_user_config') as RustConfig
  if (res) {
    config.value.baudRate = res?.baud_rate
    config.value.sendData = res.send_data
    config.value.hexSend = res.hex_send
    config.value.hexDisplay = res.hex_display
    config.value.autoScroll = res.auto_scroll
    config.value.saveSendMsg = res.save_send_msg
    config.value.receivedTextColor = res.received_text_color
    config.value.sendTextColor = res.send_text_color
  }
})

// 监听配置变化
watch(config.value, () => {
  invoke('save_user_config', {
    config: {
      baud_rate: config.value.baudRate,
      send_data: config.value.sendData,
      hex_send: config.value.hexSend,
      hex_display: config.value.hexDisplay,
      auto_scroll: config.value.autoScroll,
      save_send_msg: config.value.saveSendMsg,
      received_text_color: config.value.receivedTextColor,
      send_text_color: config.value.sendTextColor,
    }
  })
})

addEventListener('keydown', (e) => {
  if (e.key === 'Enter' && e.altKey) {
    // Alt + Enter 发送数据
    sendToPort()
  }
})

// serial-data 事件监听
function serialDataListener() {
  let buffer = ''                         // 暂存数据
  let mergeTimer: number | null = null    // 合并定时器
  const TIMEOUT = 5                       // 合并等待时间 (ms)
  listen('serial-data', event => {
    const data = event.payload as string
    if (!data) return
    receivedCount.value += data.length
    // 清除已有定时器
    if (mergeTimer) {
      clearTimeout(mergeTimer)
    }
    // 合并到缓冲区
    buffer += data
    // 启动合并定时器
    mergeTimer = window.setTimeout(() => {
      if (config.value.hexDisplay) {
        // 十六进制显示
        const hex = buffer.split('').map((item) => {
          return item.charCodeAt(0).toString(16).padStart(2, '0').toUpperCase()
        }).join(' ')
        receivedData.value.push({
          data: hex,
          direction: 'in'
        })
      } else {
        // 普通显示
        receivedData.value.push({
          data: buffer,
          direction: 'in'
        })
      }
      // 清空缓冲区和定时器
      buffer = ''
      mergeTimer = null
      // 滚动到底部
      nextTick(() => {
        if (config.value.autoScroll && receivedContext.value) {
          const el = receivedContext.value
          el.scrollTop = el.scrollHeight
        }
      })
    }, TIMEOUT)
  })
}

// serial-error 事件监听
function serialErrorListener() {
  let lastErrorShown = false
  listen('serial-error', event => {
    const data = event.payload as string
    isPortOpen.value = false;
    if (!lastErrorShown && data) {
      ElMessage.error(`连接出错: ${data}`)
      lastErrorShown = true
      setTimeout(() => {
        lastErrorShown = false
      }, 1000)
    }
  })
}

// 扫描串口
async function scanPorts() {
  let res = await invoke("scan_ports", {});
  availablePorts.value = (JSON.parse(res as string) as PortName[]).sort((a, b) => {
    return a.name.localeCompare(b.name)
  });
}

// 串口打开和关闭
function togglePort() {
  if (!config.value.port) {
    ElMessage.warning('请选择串口')
    return
  }
  if (!isPortOpen.value) {
    // 打开串口
    invoke("open_port", {
      config: {
        port: config.value.port,
        baud_rate: config.value.baudRate,
        data_bits: config.value.dataBits,
        stop_bits: config.value.stopBits,
        parity: config.value.parity,
        timeout: 500
      }
    }).then(() => {
      isPortOpen.value = true
      // 开始监听串口数据
      invoke("start_serial_listener").then(() => {
        ElMessage.success('串口已打开')
      }).catch((err) => {
        ElMessage.error(err)
      })
    }).catch((err) => {
      ElMessage.error(err)
    })
  } else {
    // 停止监听串口数据
    invoke("stop_serial_listener").then(() => {
      // 关闭串口
      invoke("close_port").then(() => {
        isPortOpen.value = false
        ElMessage.success('串口已关闭')
      }).catch((err) => {
        ElMessage.error(err)
      })
    }).catch((err) => {
      ElMessage.error(err)
    })
  }
}

// 清空接收区
function clearReceived() {
  receivedData.value = []
  receivedCount.value = 0
  sendCount.value = 0
}

// 发送数据
function sendToPort() {
  if (!config.value.sendData) return
  sendCount.value += config.value.sendData.length
  // 16进制发送
  if (config.value.hexSend) {
    const hexArray = config.value.sendData
        .trim()
        .split(/\s+/) // 根据空格分隔
        .map(h => {
          const byte = parseInt(h, 16)
          if (isNaN(byte) || byte < 0 || byte > 255) {
            throw new Error(`无效的十六进制字节: "${h}"`)
          }
          return byte
        })
    // 发送字节数组
    invoke("write_serial", {data: hexArray}).then(() => {
    }).catch((err) => {
      ElMessage.error(err)
    })
  }
  // 普通发送
  else {
    invoke("write_serial", {
      // 将字符串转换为字节数组
      data: Array.from(new TextEncoder().encode(config.value.sendData))
    }).then(() => {
      // 发送成功
    }).catch((err) => {
      ElMessage.error(err)
    })
  }
  // 写入串口
  receivedData.value.push({
    data: config.value.sendData,
    direction: 'out'
  })
  // 滚动到底部
  nextTick(() => {
    if (config.value.autoScroll && receivedContext.value) {
      const el = receivedContext.value
      el.scrollTop = el.scrollHeight
    }
  })
  // 清空发送区
  if (!config.value.saveSendMsg) {
    config.value.sendData = ''
  }
}

// 处理输入事件 16 进制发送
function sendDataInputEvent(value: string) {
  if (config.value.hexSend) {
    // 移除所有非十六进制字符（保留0-9, A-F, a-f）
    const hexOnly = value.replace(/[^0-9a-fA-F]/g, '').toUpperCase()
    // 更新输入框绑定值（比如 sendData.value）
    config.value.sendData = hexOnly.match(/.{1,2}/g)?.join(' ') ?? ''
  }
}
</script>

<style scoped>
.container {
  width: 100%;
  height: 100vh;
  padding: 12px;
  display: flex;
}

.container-config {
  width: 240px;
  height: 100%;
  margin-right: 20px;
  margin-top: -15px;
}

.container-received {
  width: 100%;
  height: 100%;
  flex: 1;
  display: flex;
  flex-direction: column;
}

.send-data-button {
  width: 160px;
  height: 94px;
  margin-left: 3px;
}

.left {
  float: left;
}

.right {
  float: right;
}

:deep(.locked-textarea textarea) {
  resize: none;
}
</style>
