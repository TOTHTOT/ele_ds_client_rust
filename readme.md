# 电子桌搭的rust版本

## 带实现功能
1. [ ] ota升级.
2. [ ] 版本回滚.
3. [ ] 低功耗. 
4. [ ] wifi以及http客户端, 和服务器通信收发数据. 
5. [ ] 驱动屏幕以及gui. 
6. [ ] 文件系统. 
7. [ ] 板子上的传感器. 
8. [ ] 终端, 实现基本控制.

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