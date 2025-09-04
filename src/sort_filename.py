def sort_file_content(input_file_path, output_file_path):
    try:
        # 读取文件内容
        with open(input_file_path, 'r', encoding='utf-8') as file:
            lines = file.readlines()
        
        # 去除每行末尾的换行符并排序
        stripped_lines = [line.strip() for line in lines]
        sorted_lines = sorted(stripped_lines)
        
        # 将排序后的内容写入新文件
        with open(output_file_path, 'w', encoding='utf-8') as file:
            for line in sorted_lines:
                file.write(line + '\n')
        
        print(f"文件已成功排序并保存到 {output_file_path}")
    
    except FileNotFoundError:
        print(f"错误: 找不到文件 {input_file_path}")
    except Exception as e:
        print(f"发生错误: {e}")

if __name__ == "__main__":
    input_file = 'tmp2.txt'    # 替换为你的输入文件路径
    output_file = 'sort_tmp2.txt'  # 替换为你的输出文件路径
    sort_file_content(input_file, output_file)
