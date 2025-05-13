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
        <el-checkbox v-model="hexSend">十六进制发送</el-checkbox>
      </div>
      <div>
        <span style="font-weight: 500; font-size: 14px; color: #606266; margin-right: 100px;">发送字符串颜色</span>
        <el-color-picker v-model="sendTextColor" />
      </div>
      <el-divider content-position="left">接收配置</el-divider>
      <div>
        <el-checkbox v-model="hexDisplay">十六进制显示</el-checkbox>
      </div>
      <div>
        <span style="font-weight: 500; font-size: 14px; color: #606266; margin-right: 100px;">接收字符串颜色</span>
        <el-color-picker v-model="receivedTextColor" />
      </div>
    </div>

    <!-- 发送和接收 -->
    <div class="container-received">
      <!-- 接收数据 -->
      <div ref="receivedContext"
           style="flex: 1; border: 1px solid #DCDFE6; border-radius: 5px; margin-bottom: 3px; padding: 10px; overflow: auto">
        <p v-for="item in receivedData">
          <span v-if="item.direction == 'in'" :style="{color: receivedTextColor}">« {{ item.data }}</span>
          <span v-else :style="{color: sendTextColor}">» {{ item.data }}</span>
        </p>
      </div>
      <!-- 发送数据 -->
      <div style="height: 140px">
        <div style="display: flex; flex-direction: row">
          <el-input v-model="sendData" type="textarea" :rows="4" class="locked-textarea" style="flex: 1"
                    @input="sendDataInputEvent"/>
          <el-button type="primary" @click="sendToPort" :disabled="!isPortOpen" class="send-data-button">发送
          </el-button>
        </div>
        <div style="margin-top: 8px; padding: 0 10px">
          <span class="left" style="margin-right: 20px">发送: {{ sendCount }}</span>
          <span class="left">接收: {{ receivedCount }}</span>
          <el-button class="right" type="danger" @click="clearReceived">清空接收区</el-button>
          <el-checkbox class="right" v-model="autoScroll" style="margin-right: 8px">自动滚动</el-checkbox>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import {ElMessage} from 'element-plus'
import {nextTick, onMounted, ref} from 'vue'
import {invoke} from "@tauri-apps/api/core";
import {listen} from '@tauri-apps/api/event'

// 串口配置
const config = ref({
  port: '',
  baudRate: 9600,
  dataBits: 8,
  stopBits: 1,
  parity: 'none',
})

interface PortName {
  name: string,
  port_type: string
}

interface Message {
  data: string
  direction: 'in' | 'out'
}

const isPortOpen = ref(false) // 串口是否打开
const availablePorts = ref<PortName[]>([]) // 可用串口列表
const baudRates = [9600, 19200, 38400, 57600, 115200] // 波特率列表
const receivedData = ref<Message[]>([]) // 接收数据
const sendData = ref('') // 发送数据
const hexDisplay = ref(false) // 十六进制显示
const hexSend = ref(false) // 十六进制发送
const autoScroll = ref(true) // 自动滚动
const receivedCount = ref(0) // 接收数据计数
const sendCount = ref(0) // 发送数据计数
const receivedContext = ref() // 接收数据容器
const sendTextColor = ref('#FFAA00')
const receivedTextColor = ref('#000000')

onMounted(() => {
  let buffer = ''                         // 暂存数据
  let mergeTimer: number | null = null   // 合并定时器
  const TIMEOUT = 5                     // 合并等待时间 (ms)
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
      if (hexDisplay.value) {
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
        if (autoScroll.value && receivedContext.value) {
          const el = receivedContext.value
          el.scrollTop = el.scrollHeight
        }
      })
    }, TIMEOUT)
  })

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
})

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
  if (!sendData.value) return
  sendCount.value += sendData.value.length
  if (hexSend.value) {
    const hexArray = sendData.value
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
  } else {
    // 普通发送
    invoke("write_serial", {data: Array.from(sendData.value, c => c.charCodeAt(0))}).then(() => {
    }).catch((err) => {
      ElMessage.error(err)
    })
  }
  // 写入串口
  receivedData.value.push({
    data: sendData.value,
    direction: 'out'
  })
  // 滚动到底部
  nextTick(() => {
    if (autoScroll.value && receivedContext.value) {
      const el = receivedContext.value
      el.scrollTop = el.scrollHeight
    }
  })
}

function sendDataInputEvent(value: string) {
  if (hexSend.value) {
    // 移除所有非十六进制字符（保留0-9, A-F, a-f）
    const hexOnly = value.replace(/[^0-9a-fA-F]/g, '').toUpperCase()
    // 更新输入框绑定值（比如 sendData.value）
    sendData.value = hexOnly.match(/.{1,2}/g)?.join(' ') ?? ''
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
