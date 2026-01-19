from PIL import Image

def convert_png_to_mono_bmp(input_path, output_path):
    try:
        # 1. 加载 PNG
        img = Image.open(input_path)

        # 2. 如果有透明通道，先填充为白色背景 (墨水屏背景通常是白色)
        if img.mode in ('RGBA', 'LA') or (img.mode == 'P' and 'transparency' in img.info):
            # 创建一个白色背景
            background = Image.new('RGB', img.size, (255, 255, 255))
            # 获取 Alpha 通道作为遮罩
            mask = img.split()[-1]
            background.paste(img, mask=mask)
            img = background
        else:
            img = img.convert('RGB')

        # 3. 智能缩放逻辑
        max_width, max_height = 282, 91

        # 获取当前尺寸
        current_width, current_height = img.size

        if current_width > max_width or current_height > max_height:
            print(f"检测到图像尺寸 ({current_width}x{current_height}) 大于显示限制，正在进行等比缩放...")
            # thumbnail 会就地修改图像，且保证比例不变，且不会放大图像
            img.thumbnail((max_width, max_height), Image.Resampling.LANCZOS)
        else:
            print(f"图像尺寸 ({current_width}x{current_height}) 符合范围，保持原样。")

        # 4. 关键：转为单色 (1-bit)
        # 使用 dither=Image.Dither.NONE 可以关闭抖动，让线条更干净
        # 如果你想要素描感，可以开启抖动 (默认开启)
        img_mono = img.convert('1', dither=Image.Dither.NONE)

        # 5. 保存为 BMP
        img_mono.save(output_path, "BMP")

        print(f"转换成功: {output_path}")
        print(f"最终保存尺寸: {img_mono.size}")
        print(f"文件位深: 1-bit (Monochrome)")

    except Exception as e:
        print(f"转换失败: {e}")

if __name__ == "__main__":
    convert_png_to_mono_bmp("test.png", "test.bmp")