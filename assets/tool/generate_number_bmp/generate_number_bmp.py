from PIL import Image, ImageDraw, ImageFont
import os

def generate_digit_bmps(font_path, output_dir, size=(60, 110), font_size=100):
    """
    font_path: 字体文件路径 (如 "arial.ttf")
    output_dir: 存放生成的 bmp 的文件夹
    size: 目标图片尺寸 (宽, 高)
    font_size: 字体渲染大小
    """
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    # 加载字体
    try:
        font = ImageFont.truetype(font_path, font_size)
    except IOError:
        print("无法加载字体，请检查路径！")
        return

    digits = "0123456789:" # 包含冒号

    for char in digits:
        # 1. 创建一个 1-bit (黑白) 的画布，初始为白色 (1)
        # '1' 表示 1-bit 像素，黑白二色
        img = Image.new('1', size, color=1)
        draw = ImageDraw.Draw(img)

        # 2. 计算文字位置使其居中
        # 获取文字的边界框
        left, top, right, bottom = draw.textbbox((0, 0), char, font=font)
        text_width = right - left
        text_height = bottom - top

        position = ((size[0] - text_width) // 2 - left,
                    (size[1] - text_height) // 2 - top)

        # 3. 渲染文字，颜色为黑色 (0)
        draw.text(position, char, font=font, fill=0)

        # 4. 保存为 BMP
        file_name = "colon.bmp" if char == ":" else f"{char}.bmp"
        img.save(os.path.join(output_dir, file_name))
        print(f"已生成: {file_name} ({size[0]}x{size[1]})")

# 使用示例
# 请确保当前目录下有一个字体文件，或者指向系统路径
font_file = "./ARIALNBI.TTF" # Windows 黑体示例
generate_digit_bmps(font_file, "./output_digits", size=(20, 40), font_size=40)