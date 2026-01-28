# 电子桌搭的rust版本

## 带实现功能
1. [x] ota升级.
2. [x] 低功耗. 
   - 目前决定每次进入低功耗都是深度睡眠, 每次唤醒后在跑完流程后直接刷新屏幕, 当屏幕乱码时手动按下按键全刷屏幕.
3. [x] wifi以及http客户端, 和服务器通信收发数据. 
4. [x] 驱动屏幕以及gui, 屏幕芯片是: SSD1680, 分辨率: 296*128. 
5. [x] 文件系统. 
6. [x] 板子上的传感器. 
7. [ ] 终端, 实现基本控制.
8. [x] 开启http客户端映射文件系统内容.

## 已知问题
1. [x] 刷新屏幕时瞬时电流高达1.5a, 会导致充电芯片重启, 三极管坏了.
2. [ ] 屏幕位置需要往按键方向移动5mm.
3. [ ] 读取电量问题, ly6806通过三根线输出pwm控制4个led, 这导致单片机没法直接读取io获取电量.

## 备注
- 执行这个命令可以生成bin文件用于ota升级.
    ```shell
  espflash save-image --chip esp32s3 target/xtensa-esp32s3-espidf/release/ele_ds_client_rust "./asset/upgrade_file/$(date +'%Y-%m-%d %H:%M:%S').bin"
    ```
- 如果出现烧入程序但是一直运行的是上个版本的软件就要清除flash, 由于升级时更新了`otadata`分区导致的.
    ```shell
    espflash erase-flash
    ```
  
- 重新监控串口
  ```shell
  espflash monitor --port /dev/ttyUSB0
  ```
  
- 使用内容的psram
  - 配置好了psram但是由于电压错误导致不能正确和psram通信, 这是由于efuse配置问题导致的, 电压锁死3.3V但是目标是1.8V, 现在由于硬件情况不能使用psram了.
  ```shell
  # 读取efuse参数
  espefuse.py summary --port /dev/ttyUSB1
  ```
  ```text
  #
  # ESP PSRAM
  #
  CONFIG_SPIRAM=y
    
  #
  # SPI RAM config
  #
  # CONFIG_SPIRAM_MODE_QUAD is not set
  CONFIG_SPIRAM_MODE_OCT=y
  CONFIG_SPIRAM_TYPE_AUTO=y
  # CONFIG_SPIRAM_TYPE_ESPPSRAM64 is not set
  CONFIG_SPIRAM_ALLOW_STACK_EXTERNAL_MEMORY=y
  CONFIG_SPIRAM_CLK_IO=30
  CONFIG_SPIRAM_CS_IO=26
  # CONFIG_SPIRAM_XIP_FROM_PSRAM is not set
  # CONFIG_SPIRAM_FETCH_INSTRUCTIONS is not set
  # CONFIG_SPIRAM_RODATA is not set
  # CONFIG_SPIRAM_SPEED_80M is not set
  CONFIG_SPIRAM_SPEED_40M=y
  CONFIG_SPIRAM_SPEED=40
  # CONFIG_SPIRAM_ECC_ENABLE is not set
  CONFIG_SPIRAM_BOOT_INIT=y
  # CONFIG_SPIRAM_IGNORE_NOTFOUND is not set
  # CONFIG_SPIRAM_USE_MEMMAP is not set
  # CONFIG_SPIRAM_USE_CAPS_ALLOC is not set
  CONFIG_SPIRAM_USE_MALLOC=y
  # CONFIG_SPIRAM_MEMTEST is not set
  CONFIG_SPIRAM_MALLOC_ALWAYSINTERNAL=16384
  # CONFIG_SPIRAM_TRY_ALLOCATE_WIFI_LWIP is not set
  CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=32768
  # CONFIG_SPIRAM_ALLOW_BSS_SEG_EXTERNAL_MEMORY is not set
  # CONFIG_SPIRAM_ALLOW_NOINIT_SEG_EXTERNAL_MEMORY is not set
  # end of SPI RAM config
  # end of ESP PSRAM
  ```
    
- 使用PUT上传文件, 会直接覆盖原先文件!!!
  ```shell
  curl -X PUT --data-binary "@src/main.rs" http://10.22.110.117/fat/A/B/C/new_file.txt
  ```
  
- 发送图片资源
```shell
cd assets/tool/generate_number_bmp
for file in time_num_data/*.bmp; do
  filename=$(basename "$file")
  echo "正在上传: $filename ..."
  curl -X PUT --data-binary "@$file" "http://192.168.137.183/fat/system/tmd/$filename"
done
```